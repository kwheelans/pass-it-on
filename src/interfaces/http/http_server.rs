use crate::interfaces::http::{Version, BASE_PATH, NOTIFICATION_PATH, VERSION_PATH};
use crate::notifications::Notification;
use crate::LIB_LOG_TARGET;
use log::{info, warn};
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use tokio::sync::{mpsc, watch};
use warp::http::StatusCode;
use warp::Filter;

const DEFAULT_BODY_LIMIT: u64 = 1024 * 1024;

pub(super) async fn start_monitoring<P: AsRef<Path>>(
    tx: mpsc::Sender<String>,
    shutdown: watch::Receiver<bool>,
    socket: SocketAddr,
    tls: bool,
    tls_cert_path: Option<P>,
    tls_key_path: Option<P>,
) {
    let mut shutdown_rx = shutdown.clone();
    let sender = warp::any().map(move || tx.clone());
    let server_version = warp::any().map(Version::new);

    let notifications = warp::post()
        .and(warp::path(BASE_PATH))
        .and(warp::path(NOTIFICATION_PATH))
        .and(notification_json_body())
        .and(sender)
        .and_then(receive_notification);

    let version = warp::get()
        .and(warp::path(BASE_PATH))
        .and(warp::path(VERSION_PATH))
        .and(warp::path::end())
        .and(server_version)
        .and_then(send_json_response);

    let routes = notifications.or(version);

    info!(target: LIB_LOG_TARGET, "Setting up Interface: HttpSocket on -> {} | TLS Enabled -> {}", socket, tls);
    match tls {
        true => {
            let (_address, server) = warp::serve(routes)
                .tls()
                .cert_path(tls_cert_path.unwrap().as_ref())
                .key_path(tls_key_path.unwrap().as_ref())
                .bind_with_graceful_shutdown(socket, async move {
                    shutdown_rx.changed().await.ok().unwrap_or_default();
                });
            server.await;
        }
        false => {
            let (_address, server) = warp::serve(routes).bind_with_graceful_shutdown(socket, async move {
                shutdown_rx.changed().await.ok().unwrap_or_default();
            });
            server.await;
        }
    };
}

async fn receive_notification(
    notification: Notification,
    tx: mpsc::Sender<String>,
) -> Result<impl warp::Reply, Infallible> {
    match tx.send(notification.to_json().unwrap_or_default()).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            warn!(target: LIB_LOG_TARGET, "bad JSON received from http socket: {}", e);
            Ok(StatusCode::BAD_REQUEST)
        }
    }
}

async fn send_json_response<R: Serialize>(response: R) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&response))
}

fn notification_json_body() -> impl Filter<Extract = (Notification,), Error = warp::Rejection> + Clone {
    warp::any().and(warp::body::content_length_limit(DEFAULT_BODY_LIMIT)).and(warp::body::json())
}
