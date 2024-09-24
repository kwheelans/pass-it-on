use crate::interfaces::pipe::cleanup_pipe;
use crate::{Error, LIB_LOG_TARGET};
use tracing::warn;
use std::path::Path;
use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio::sync::watch;

pub async fn read_pipe<P: AsRef<Path>>(
    path: P,
    interface_tx: mpsc::Sender<String>,
    shutdown: watch::Receiver<bool>,
) -> Result<(), Error> {
    let mut shutdown_rx = shutdown.clone();
    let tx = interface_tx.clone();
    loop {
        let mut pipe_rx = tokio::net::unix::pipe::OpenOptions::new().open_receiver(path.as_ref())?;

        tokio::select! {
            readable = pipe_rx.readable() => {
                match readable {
                                Ok(_) => {
               let mut read_string = String::new();
               match pipe_rx.read_to_string(&mut read_string).await {
                   Ok(_) => {
                       if let Err(e) = tx.send(read_string).await { warn!(target: LIB_LOG_TARGET, "{}", e) }
                   },
                   Err(e) => {
                       warn!(target: LIB_LOG_TARGET, "{}", e);
                       return Err(e.into())
                   }
               }
           },
           Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
               warn!(target: LIB_LOG_TARGET, "{}", e);
               continue;
           },

           Err(e) => {
               warn!(target: LIB_LOG_TARGET, "{}", e);
               return Err(e.into())
           }
                }
            }

            _ = shutdown_rx.changed() => {
                break;
            }
        }
    }
    cleanup_pipe(path.as_ref()).await?;
    Ok(())
}
