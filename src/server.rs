use crate::configuration::ServerConfiguration;
use crate::endpoints::{setup_endpoints, EndpointChannel};
use crate::interfaces::setup_server_interfaces;
use crate::notifications::{Notification, ValidatedNotification};
use crate::shutdown::listen_for_shutdown;
use crate::{Error, LIB_LOG_TARGET};
use log::{debug, info, warn};
use tokio::sync::{mpsc, watch};

const DEFAULT_WAIT_FOR_SHUTDOWN_SECS: u64 = 2;

/// Start the server with provided `ServerConfiguration`.
///
/// Server listens for shutdown signals SIGTERM & SIGINT on Unix or CTRL-BREAK and CTRL-C on Windows.
pub async fn start_server(
    server_config: ServerConfiguration,
    wait_for_shutdown_secs: Option<u64>,
) -> Result<(), Error> {
    // Setup channels
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (interface_tx, interface_rx) = mpsc::channel(100);

    // Start monitoring the configured interfaces
    let interfaces = server_config.interfaces();
    setup_server_interfaces(interfaces, interface_tx.clone(), shutdown_rx.clone()).await?;

    // Setup endpoints to receive messages
    let endpoints = server_config.endpoint_channels();
    setup_endpoints(endpoints.clone(), shutdown_rx.clone()).await?;

    // Monitor for messages on the interface channel
    tokio::spawn(async move {
        process_incoming_notifications(interface_rx, endpoints).await;
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

async fn process_incoming_notifications(mut msg_rx: mpsc::Receiver<String>, endpoints: Vec<EndpointChannel>) {
    info!(target: LIB_LOG_TARGET, "Processing Notifications");

    while let Some(msg) = msg_rx.recv().await {
        let notifications = Notification::from_json_multi(msg.as_str());

        for notification in notifications {
            match notification {
                Ok(note) => {
                    for endpoint in &endpoints {
                        for (sub_name, keys) in endpoint.keys() {
                            if note.validate_set(keys) {
                                let channel = endpoint.channel_sender();
                                match channel.send(ValidatedNotification::new(sub_name.to_string(), note.message())) {
                                    Ok(ok) => {
                                        debug!(target: LIB_LOG_TARGET, "Message sent to endpoint. Subscribers: {}", ok)
                                    }
                                    Err(e) => warn!(
                                        target: LIB_LOG_TARGET,
                                        "Error sending validated message to endpoint: {}", e
                                    ),
                                };
                            }
                        }
                    }
                }

                Err(e) => warn!(target: LIB_LOG_TARGET, "Notification processing error: {}", e),
            }
        }
    }
}
