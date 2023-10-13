use crate::LIB_LOG_TARGET;
use log::{error, info};
use std::time::Duration;
use tokio::sync::watch;

#[cfg(unix)]
pub(crate) async fn listen_for_shutdown(
    shutdown_tx: watch::Sender<bool>,
    shutdown: Option<watch::Receiver<bool>>,
    seconds_to_wait: u64,
) {
    use tokio::signal::unix::{signal, SignalKind};
    // Listen for SIGTERM and SIGINT to know when shutdown
    let mut sigterm = signal(SignalKind::terminate()).expect("unable to listen for terminate signal");
    let mut sigint = signal(SignalKind::interrupt()).expect("unable to listen for interrupt signal");

    if let Some(mut shutdown_rx) = shutdown {
        tokio::select! {
        _ = sigterm.recv() => info!(target: LIB_LOG_TARGET, "Received SIGTERM."),
        _ = sigint.recv() => info!(target: LIB_LOG_TARGET, "Received SIGINT."),
        _ = shutdown_rx.changed() => info!(target: LIB_LOG_TARGET, "Received shutdown channel signal."),
        }
    } else {
        tokio::select! {
        _ = sigterm.recv() => info!(target: LIB_LOG_TARGET, "Received SIGTERM."),
        _ = sigint.recv() => info!(target: LIB_LOG_TARGET, "Received SIGINT."),
        }
    }

    // Send shutdown signal
    if let Err(error) = shutdown_tx.send(true) {
        error!(target: LIB_LOG_TARGET, "Unable to send shutdown signal: {}", error)
    }

    info!(target: LIB_LOG_TARGET, "Starting Shutdown");
    // Allow time for cleanup
    tokio::time::sleep(Duration::from_secs(seconds_to_wait)).await;
    info!(target: LIB_LOG_TARGET, "Shutdown Complete")
}

#[cfg(windows)]
pub(crate) async fn listen_for_shutdown(
    shutdown_tx: watch::Sender<bool>,
    shutdown: Option<watch::Receiver<bool>>,
    seconds_to_wait: u64,
) {
    use tokio::signal::windows::{ctrl_break, ctrl_c};
    // Listen for CTRL-C and CTRL-BREAK to know when shutdown
    let mut sig_ctrl_break = ctrl_break().expect("unable to listen for ctrl-break signal");
    let mut sig_ctrl_c = ctrl_c().expect("unable to listen for ctrl-c signal");

    if let Some(mut shutdown_rx) = shutdown {
        tokio::select! {
        _ = sig_ctrl_break.recv() => info!(target: LIB_LOG_TARGET, "Received CTRL-BREAK."),
        _ = sig_ctrl_c.recv() => info!(target: LIB_LOG_TARGET, "Received CTRL-C."),
        _ = shutdown_rx.changed() => info!(target: LIB_LOG_TARGET, "Received shutdown channel signal."),
        }
    } else {
        tokio::select! {
            _ = sig_ctrl_break.recv() => info!(target: LIB_LOG_TARGET, "Received CTRL-BREAK."),
            _ = sig_ctrl_c.recv() => info!(target: LIB_LOG_TARGET, "Received CTRL-C."),
        }
    }

    // Send shutdown signal
    if let Err(error) = shutdown_tx.send(true) {
        error!(target: LIB_LOG_TARGET, "Unable to send shutdown signal: {}", error)
    }

    info!(target: LIB_LOG_TARGET, "Starting Shutdown");
    // Allow time for cleanup
    tokio::time::sleep(Duration::from_secs(seconds_to_wait)).await;
    info!(target: LIB_LOG_TARGET, "Shutdown Complete")
}
