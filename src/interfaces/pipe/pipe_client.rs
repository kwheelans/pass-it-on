use crate::notifications::Notification;
use crate::Error;
use tracing::error;
use std::path::Path;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::sync::{broadcast, watch};

pub async fn write_pipe<P: AsRef<Path>>(
    path: P,
    mut msg_rx: broadcast::Receiver<Notification>,
    shutdown: watch::Receiver<bool>,
) -> Result<(), Error> {
    let mut shutdown_rx = shutdown.clone();

    loop {
        let mut pipe_tx = tokio::net::unix::pipe::OpenOptions::new().open_sender(path.as_ref())?;
        tokio::select! {
            msg = msg_rx.recv() => {
                match msg {
                    Ok(message) => {
                        let msg_text = get_string(message)?;
                        match pipe_tx.writable().await {
                            Ok(_) => pipe_tx.write_all(msg_text.as_bytes()).await?,
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                            Err(e) => {
                                error!("{}", e);
                                return Err(e.into())
                            },
                        }
                    },
                    Err(error) => {
                        error!("Broadcast Receiver Error: {}", error);
                        break;
                    }
                }
            }

            _ = shutdown_rx.changed() => {
                 break;
                }
        }
    }
    Ok(())
}

fn get_string(note: Notification) -> Result<String, Error> {
    note.to_json()
}
