use crate::interfaces::http::{Version, BASE_PATH, NOTIFICATION_PATH, VERSION_PATH};
use crate::notifications::Notification;
use crate::LIB_LOG_TARGET;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tracing::{debug, error, info, trace, warn};

const GRACE_PERIOD: Duration = Duration::from_secs(1);

pub(super) async fn start_monitoring<P: AsRef<Path>>(
    tx: mpsc::Sender<String>,
    shutdown: watch::Receiver<bool>,
    socket: SocketAddr,
    tls: bool,
    tls_cert_path: Option<P>,
    tls_key_path: Option<P>,
) {
    let handle = axum_server::Handle::new();
    tokio::spawn(shutdown_server(handle.clone(), shutdown));

    let routes = Router::new()
        .route(format!("/{}/{}", BASE_PATH, VERSION_PATH).as_str(), get(version_handler))
        .route(format!("/{}/{}", BASE_PATH, NOTIFICATION_PATH).as_str(), post(notification_handler))
        .with_state(tx);

    info!(target: LIB_LOG_TARGET, "Setting up Interface: HttpSocket on -> {} | TLS Enabled -> {}", socket, tls);
    match tls {
        true => {
            let config = RustlsConfig::from_pem_file(tls_cert_path.unwrap(), tls_key_path.unwrap()).await.unwrap();
            axum_server::bind_rustls(socket, config).serve(routes.into_make_service()).await.unwrap();
        }
        false => {
            axum_server::bind(socket).handle(handle).serve(routes.into_make_service()).await.unwrap();
        }
    };
}

async fn version_handler() -> Json<Version> {
    Json(Version::new())
}

async fn notification_handler(
    State(tx): State<mpsc::Sender<String>>,
    Json(notification): Json<Notification>,
) -> StatusCode {
    trace!(target: LIB_LOG_TARGET, "HTTP server received {:?}", notification);
    match tx.send(notification.to_json().unwrap_or_default()).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            warn!(target: LIB_LOG_TARGET, "bad JSON received from http socket: {}", e);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn shutdown_server(handle: axum_server::Handle, mut shutdown: watch::Receiver<bool>) {
    match shutdown.changed().await {
        Ok(_) => {
            debug!(target: LIB_LOG_TARGET, "http_server starting graceful shutdown");
            handle.graceful_shutdown(Some(GRACE_PERIOD));
        }
        Err(e) => {
            error!(target: LIB_LOG_TARGET, "Shutdown Receive Error: {}", e);
        }
    }
}
