use crate::notifications::Notification;
use crate::LIB_LOG_TARGET;
use log::{debug, error, trace, warn};
use reqwest::Client;
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::{broadcast, watch};

pub(super) async fn start_sending(
    interface_rx: broadcast::Receiver<Notification>,
    shutdown: watch::Receiver<bool>,
    url: &str,
) {
    let mut rx = interface_rx.resubscribe();
    let client = Client::new();

    loop {
        if *shutdown.borrow() {
            trace!(target: LIB_LOG_TARGET, "HTTP Client start_sending received shutdown");
            break;
        }

        match rx.try_recv() {
            Ok(message) => {
                debug!(target: LIB_LOG_TARGET, "HTTP Client received message");
                let response = client.post(url).body(message.to_json().unwrap_or_default()).send().await;
                match response {
                    Ok(ok) => debug!(
                        target: LIB_LOG_TARGET,
                        "HTTP Client Response - status: {} url: {}",
                        ok.status(),
                        ok.url()
                    ),
                    Err(error) => warn!(target: LIB_LOG_TARGET, "HTTP Client Response Error: {}", error),
                }
            }

            Err(error) => match error {
                TryRecvError::Empty => {
                    trace!(target: LIB_LOG_TARGET, "HTTP Client Broadcast Receiver: {}", error);
                }
                TryRecvError::Closed => {
                    error!(target: LIB_LOG_TARGET, "HTTP Client Broadcast Receiver Error: {}", error);
                    break;
                }
                TryRecvError::Lagged(skipped) => {
                    warn!(target: LIB_LOG_TARGET, "Broadcast Receiver: {} messages skipped: {}", error, skipped);
                }
            },
        }
    }
}
