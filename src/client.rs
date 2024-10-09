use crate::configuration::ClientConfiguration;
use crate::interfaces::{setup_client_interfaces, NANOSECOND, SECOND};
use crate::notifications::{ClientReadyMessage, Key, Notification};
use crate::shutdown::listen_for_shutdown;
use crate::{Error, CHANNEL_BUFFER};
use tracing::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};
use tokio::sync::watch::Receiver;
use tokio::sync::{broadcast, mpsc, watch};

const DEFAULT_WAIT_FOR_SHUTDOWN_SECS: u64 = 2;

/// Start the client with provided [`ClientConfiguration`] and `Receiver<ClientReadyMessage>` channel.
///
/// Client listens for shutdown signals SIGTERM & SIGINT on Unix or CTRL-BREAK and CTRL-C on Windows.
/// Also accepts a `Option<tokio::sync::watch::Receiver<bool>>` to shut down the client in addition to
/// system signals.
pub async fn start_client(
    client_config: ClientConfiguration,
    notification_rx: mpsc::Receiver<ClientReadyMessage>,
    shutdown: Option<Receiver<bool>>,
    wait_for_shutdown_secs: Option<u64>,
) -> Result<(), Error> {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (interface_tx, interface_rx) = broadcast::channel(CHANNEL_BUFFER);
    let key = client_config.key().clone();

    // Setup interfaces to send notifications to
    let interfaces = client_config.interfaces();
    setup_client_interfaces(interfaces, interface_rx, shutdown_rx.clone()).await?;

    // Monitor for incoming notifications
    tokio::spawn(async move {
        receive_notifications(notification_rx, interface_tx, shutdown_rx.clone(), key).await;
    });

    // Shutdown
    listen_for_shutdown(shutdown_tx, shutdown, wait_for_shutdown_secs.unwrap_or(DEFAULT_WAIT_FOR_SHUTDOWN_SECS)).await;

    Ok(())
}

/// Start the client with provided [`ClientConfiguration`] and `Arc<Mutex<Vec<ClientReadyMessage>>>`.
///
/// Client listens for shutdown signals SIGTERM & SIGINT  on Unix or CTRL-BREAK and CTRL-C on Windows.
/// Also accepts a `Option<tokio::sync::watch::Receiver<bool>>` to shutdown the client in addition to
/// system signals.
pub async fn start_client_arc(
    client_config: ClientConfiguration,
    notifications: Arc<Mutex<Vec<ClientReadyMessage>>>,
    shutdown: Option<Receiver<bool>>,
    wait_for_shutdown_secs: Option<u64>,
) -> Result<(), Error> {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (interface_tx, interface_rx) = broadcast::channel(CHANNEL_BUFFER);
    let key = client_config.key().clone();

    // Setup interfaces to send notifications to
    let interfaces = client_config.interfaces();
    setup_client_interfaces(interfaces, interface_rx, shutdown_rx.clone()).await?;

    // Monitor for incoming notifications
    tokio::spawn(async move {
        receive_notifications_arc(notifications, interface_tx, shutdown_rx.clone(), key).await;
    });

    // Shutdown
    listen_for_shutdown(shutdown_tx, shutdown, wait_for_shutdown_secs.unwrap_or(DEFAULT_WAIT_FOR_SHUTDOWN_SECS)).await;

    Ok(())
}

async fn receive_notifications(
    mut notification_rx: mpsc::Receiver<ClientReadyMessage>,
    interface_tx: broadcast::Sender<Notification>,
    shutdown: Receiver<bool>,
    key: Key,
) {
    info!("Client waiting for notifications");

    let mut shutdown_rx = shutdown.clone();
    loop {
        tokio::select! {
            msg = notification_rx.recv() => {
                if let Some(client_ready_msg) = msg {
                    let notification = client_ready_msg.to_notification(&key);
                    debug!("Client Sending Notification: {:?}", notification);
                    match interface_tx.send(notification) {
                        Ok(ok) => debug!("Message passed to client {} interfaces", ok),
                        Err(error) => {
                            error!("Client broadcast channel send error: {}", error);
                            break;
                        },
                    }
                }
            }

            _ = shutdown_rx.changed() => {
                trace!("Shutdown receive_notifications");
                 break;
                }

            _ = tokio::time::sleep(SECOND) => {
                trace!("Sleep timeout reached for receive_notifications");
            }
        }
        tokio::time::sleep(NANOSECOND).await;
    }
}

async fn receive_notifications_arc(
    notifications: Arc<Mutex<Vec<ClientReadyMessage>>>,
    interface_tx: broadcast::Sender<Notification>,
    shutdown: Receiver<bool>,
    key: Key,
) {
    info!("Client waiting for notifications");

    let mut shutdown_rx = shutdown.clone();
    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                trace!("Shutdown receive_notifications_arc");
                 break;
                }

            _ = tokio::time::sleep(SECOND) => {
                trace!("Sleep timeout reached for receive_notifications_arc");
            }
        }

        let messages: Vec<ClientReadyMessage> = notifications.lock().unwrap().drain(0..).collect();

        if !messages.is_empty() {
            for client_ready_msg in messages {
                let notification = client_ready_msg.to_notification(&key);
                debug!("Client attempting to send Notification: {:?}", notification);

                match interface_tx.send(notification) {
                    Ok(ok) => debug!("Message passed to client interfaces: {}", ok),
                    Err(error) => warn!("Client broadcast channel send error: {}", error),
                }
            }
        }
        tokio::time::sleep(NANOSECOND).await;
    }
}
