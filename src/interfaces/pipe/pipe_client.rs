use crate::notifications::Notification;
use crate::{Error, LIB_LOG_TARGET};
use log::{error, warn};
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
                                error!(target: LIB_LOG_TARGET, "{}", e);
                                return Err(Error::IOError(e))
                            },
                        }
                    },
                    Err(error) => warn!(target: LIB_LOG_TARGET, "{}", error),
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
