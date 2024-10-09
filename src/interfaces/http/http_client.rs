use crate::interfaces::{NANOSECOND, SECOND};
use crate::notifications::Notification;
use reqwest::Client;
use tokio::sync::{broadcast, watch};
use tracing::{debug, error, trace, warn};

pub(super) async fn start_sending(
    interface_rx: broadcast::Receiver<Notification>,
    shutdown: watch::Receiver<bool>,
    url: &str,
) {
    let mut shutdown_rx = shutdown.clone();
    let mut rx = interface_rx.resubscribe();
    let client = Client::builder().use_rustls_tls().build().expect("unable to create client");

    loop {
        tokio::select! {
            received = rx.recv() => {
                match received {
                    Ok(message) => {
                        let response = client.post(url)
                        .json(&message)
                        .send().await;
                        match response {
                            Ok(ok) => debug!("HTTP Client Response - status: {} url: {}", ok.status(), ok.url()),
                            Err(error) => warn!("HTTP Client Response Error: {}", error ),
                        }
                    },
                    Err(error) => {
                        error!("Broadcast Receiver Error: {}", error);
                        break;
                    },
                }
            }

            _ = shutdown_rx.changed() => {
                break;
            }

            _ = tokio::time::sleep(SECOND) => {
                trace!("Sleep timeout reached for http start_sending");
            }
        }
        tokio::time::sleep(NANOSECOND).await;
    }
}
