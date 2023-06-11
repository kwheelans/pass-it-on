use crate::configuration::ClientConfiguration;
use crate::interfaces::setup_client_interfaces;
use crate::notifications::Notification;
use crate::shutdown::listen_for_shutdown;
use crate::{Error, LIB_LOG_TARGET};
use log::{debug, info, warn};
use tokio::sync::{broadcast, mpsc, watch};

const DEFAULT_WAIT_FOR_SHUTDOWN_SECS: u64 = 2;

/// Start the client with provided [`ClientConfiguration`] and `Receiver<Notification>` channel.
///
/// Client listens for shutdown signals SIGTERM & SIGINT on Unix or CTRL-BREAK and CTRL-C on Windows.
pub async fn start_client(
    client_config: ClientConfiguration,
    notification_rx: mpsc::Receiver<Notification>,
    wait_for_shutdown_secs: Option<u64>,
) -> Result<(), Error> {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (interface_tx, _interface_rx) = broadcast::channel(100);

    // Setup interfaces to send notifications to
    let interfaces = client_config.interfaces();
    setup_client_interfaces(interfaces, interface_tx.subscribe(), shutdown_rx.clone()).await?;

    // Monitor for incoming notifications
    tokio::spawn(async move {
        receive_notifications(notification_rx, interface_tx).await;
    });

    // Shutdown
    let shutdown_secs = match wait_for_shutdown_secs {
        None => DEFAULT_WAIT_FOR_SHUTDOWN_SECS,
        Some(secs) => secs,
    };
    info!(target: LIB_LOG_TARGET, "Listening for shutdown signals");
    listen_for_shutdown(shutdown_tx, shutdown_secs).await;

    Ok(())
}

async fn receive_notifications(
    mut notification_rx: mpsc::Receiver<Notification>,
    interface_tx: broadcast::Sender<Notification>,
) {
    info!(target: LIB_LOG_TARGET, "Client waiting for notifications");
    while let Some(notification) = notification_rx.recv().await {
        match interface_tx.send(notification) {
            Ok(ok) => debug!(target: LIB_LOG_TARGET, "Message passed to client {} interfaces", ok),
            Err(error) => warn!(target: LIB_LOG_TARGET, "Client broadcast channel send error: {}", error),
        }
    }
}
