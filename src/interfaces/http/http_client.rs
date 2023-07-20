use crate::interfaces::{NANOSECOND, SECOND};
use crate::notifications::Notification;
use crate::LIB_LOG_TARGET;
use log::{debug, error, trace, warn};
use reqwest::Client;
use tokio::sync::{broadcast, watch};

pub(super) async fn start_sending(
    interface_rx: broadcast::Receiver<Notification>,
    shutdown: watch::Receiver<bool>,
    url: &str,
) {
    let mut shutdown_rx = shutdown.clone();
    let mut rx = interface_rx.resubscribe();
    let client = Client::new();

    loop {
        tokio::select! {
            received = rx.recv() => {
                match received {
                    Ok(message) => {
                        let response = client.post(url)
                        .body(message.to_json().unwrap_or_default())
                        .send().await;
                        match response {
                            Ok(ok) => debug!(target: LIB_LOG_TARGET,"HTTP Client Response - status: {} url: {}", ok.status(), ok.url()),
                            Err(error) => warn!(target: LIB_LOG_TARGET, "HTTP Client Response Error: {}", error ),
                        }
                    },
                    Err(error) => {
                        error!(target: LIB_LOG_TARGET, "Broadcast Receiver Error: {}", error);
                        break;
                    },
                }
            }

            _ = shutdown_rx.changed() => {
                break;
            }

            _ = tokio::time::sleep(SECOND) => {
                trace!(target: LIB_LOG_TARGET, "Sleep timeout reached for http start_sending");
            }
        }
        tokio::time::sleep(NANOSECOND).await;
    }
}
