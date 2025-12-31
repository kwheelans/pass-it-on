use crate::interfaces::http::{BASE_PATH, NOTIFICATION_PATH, VERSION_PATH, Version};
use crate::notifications::Notification;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_server::Address;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tracing::{debug, error, info, trace, warn};
use crate::Error;

const GRACE_PERIOD: Duration = Duration::from_secs(1);

pub(super) async fn start_monitoring<P: AsRef<Path>> (
    tx: mpsc::Sender<String>,
    shutdown: watch::Receiver<bool>,
    socket: SocketAddr,
    tls: bool,
    tls_cert_path: Option<P>,
    tls_key_path: Option<P>,
) -> Result<(), Error> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let handle = axum_server::Handle::new();
    tokio::spawn(shutdown_server(handle.clone(), shutdown));

    let routes = Router::new()
        .route(format!("/{}/{}", BASE_PATH, VERSION_PATH).as_str(), get(version_handler))
        .route(format!("/{}/{}", BASE_PATH, NOTIFICATION_PATH).as_str(), post(notification_handler))
        .with_state(tx);

    info!("Setting up Interface: HttpSocket on -> {} | TLS Enabled -> {}", socket, tls);
    let listener = std::net::TcpListener::bind(socket)?;
    listener.set_nonblocking(true)?;
    
    match tls {
        true => {
            let config = RustlsConfig::from_pem_file(tls_cert_path.unwrap(), tls_key_path.unwrap())
                .await?;
            axum_server::from_tcp_rustls(listener, config)?
                .serve(routes.into_make_service())
                .await?;
        }
        false => {
            axum_server::from_tcp(listener)?
                .handle(handle)
                .serve(routes.into_make_service())
                .await?
        }
    };
    Ok(())
}

async fn version_handler() -> Json<Version> {
    Json(Version::new())
}

async fn notification_handler(
    State(tx): State<mpsc::Sender<String>>,
    Json(notification): Json<Notification>,
) -> StatusCode {
    trace!("HTTP server received {:?}", notification);
    match tx.send(notification.to_json().unwrap_or_default()).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            warn!("bad JSON received from http socket: {}", e);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn shutdown_server<A: Address>(handle: axum_server::Handle<A>, mut shutdown: watch::Receiver<bool>) {
    match shutdown.changed().await {
        Ok(_) => {
            debug!("http_server starting graceful shutdown");
            handle.graceful_shutdown(Some(GRACE_PERIOD));
        }
        Err(e) => {
            error!("Shutdown Receive Error: {}", e);
        }
    }
}
