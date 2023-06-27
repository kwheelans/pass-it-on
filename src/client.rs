use crate::configuration::ClientConfiguration;
use crate::interfaces::setup_client_interfaces;
use crate::notifications::{ClientReadyMessage, Key, Notification};
use crate::shutdown::listen_for_shutdown;
use crate::{Error, LIB_LOG_TARGET};
use log::{debug, info, warn};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc, watch};

const DEFAULT_WAIT_FOR_SHUTDOWN_SECS: u64 = 2;

/// Start the client with provided [`ClientConfiguration`] and `Receiver<ClientReadyMessage>` channel.
///
/// Client listens for shutdown signals SIGTERM & SIGINT on Unix or CTRL-BREAK and CTRL-C on Windows.
pub async fn start_client(
    client_config: ClientConfiguration,
    notification_rx: mpsc::Receiver<ClientReadyMessage>,
    wait_for_shutdown_secs: Option<u64>,
) -> Result<(), Error> {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (interface_tx, interface_rx) = broadcast::channel(100);
    let key = client_config.key().clone();

    // Setup interfaces to send notifications to
    let interfaces = client_config.interfaces();
    setup_client_interfaces(interfaces, interface_rx, shutdown_rx.clone()).await?;

    // Monitor for incoming notifications
    tokio::spawn(async move {
        receive_notifications(notification_rx, interface_tx, shutdown_rx.clone(), key).await;
    });

    // Shutdown
    info!(target: LIB_LOG_TARGET, "Listening for shutdown signals");
    listen_for_shutdown(shutdown_tx, get_shutdown_wait_time(wait_for_shutdown_secs)).await;

    Ok(())
}

/// Start the client with provided [`ClientConfiguration`] and `Arc<Mutex<Vec<ClientReadyMessage>>>`.
///
/// Client listens for shutdown signals SIGTERM & SIGINT on Unix or CTRL-BREAK and CTRL-C on Windows.
pub async fn start_client_arc(
    client_config: ClientConfiguration,
    notifications: Arc<Mutex<Vec<ClientReadyMessage>>>,
    wait_for_shutdown_secs: Option<u64>,
) -> Result<(), Error> {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (interface_tx, interface_rx) = broadcast::channel(100);
    let key = client_config.key().clone();

    // Setup interfaces to send notifications to
    let interfaces = client_config.interfaces();
    setup_client_interfaces(interfaces, interface_rx, shutdown_rx.clone()).await?;

    // Monitor for incoming notifications
    tokio::spawn(async move {
        receive_notifications_arc(notifications, interface_tx, shutdown_rx.clone(), key).await;
    });

    // Shutdown
    info!(target: LIB_LOG_TARGET, "Listening for shutdown signals");
    listen_for_shutdown(shutdown_tx, get_shutdown_wait_time(wait_for_shutdown_secs)).await;

    Ok(())
}

async fn receive_notifications_arc(
    notifications: Arc<Mutex<Vec<ClientReadyMessage>>>,
    interface_tx: broadcast::Sender<Notification>,
    shutdown: watch::Receiver<bool>,
    key: Key,
) {
    info!(target: LIB_LOG_TARGET, "Client waiting for notifications");

    loop {
        if *shutdown.borrow() {
            debug!(target: LIB_LOG_TARGET, "receive_notifications_arc received shutdown");
            break;
        }

        let messages: Vec<ClientReadyMessage> = notifications.lock().unwrap().drain(0..).collect();

        if !messages.is_empty() {
            for client_ready_msg in messages {
                let notification = client_ready_msg.to_notification(&key);
                debug!(target: LIB_LOG_TARGET, "Client attempting to send Notification: {:?}", notification);

                match interface_tx.send(notification) {
                    Ok(ok) => debug!(target: LIB_LOG_TARGET, "Message passed to client interfaces: {}", ok),
                    Err(error) => warn!(target: LIB_LOG_TARGET, "Client broadcast channel send error: {}", error),
                }
            }
        }
    }
}

async fn receive_notifications(
    mut notification_rx: mpsc::Receiver<ClientReadyMessage>,
    interface_tx: broadcast::Sender<Notification>,
    shutdown: watch::Receiver<bool>,
    key: Key,
) {
    info!(target: LIB_LOG_TARGET, "Client waiting for notifications");

    let mut shutdown_rx = shutdown.clone();
    loop {
        tokio::select! {
            msg = notification_rx.recv() => {
                if let Some(client_ready_msg) = msg {
                    let notification = client_ready_msg.to_notification(&key);
                    debug!(target: LIB_LOG_TARGET, "Client Sending Notification: {:?}", notification);
                    match interface_tx.send(notification) {
                        Ok(ok) => debug!(target: LIB_LOG_TARGET, "Message passed to client {} interfaces", ok),
                        Err(error) => warn!(target: LIB_LOG_TARGET, "Client broadcast channel send error: {}", error),
                    }
                }
            }

            _ = shutdown_rx.changed() => {
                 break;
                }
        }
    }
}

fn get_shutdown_wait_time(seconds: Option<u64>) -> u64 {
    match seconds {
        None => DEFAULT_WAIT_FOR_SHUTDOWN_SECS,
        Some(secs) => secs,
    }
}
