//! Regular file [`Endpoint`] and [`EndpointConfig`] implementation
//!
//! # Configuration Example
//! ```toml
//! [[server.endpoint]]
//! type = "file"
//! path = 'path/to/file_endpoint.txt'
//! notifications = ["notification_id1", "notification_id2"]
//! ```

use crate::endpoints::{Endpoint, EndpointConfig};
use crate::notifications::{Key, ValidatedNotification};
use crate::{Error, LIB_LOG_TARGET};
use async_trait::async_trait;
use log::{info, warn};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::{broadcast, watch};

const LINE_FEED: &[u8] = "\n".as_bytes();

/// Data structure to represent the regular file [`EndpointConfig`].
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub(crate) struct FileConfigFile {
    path: String,
    notifications: Vec<String>,
}

/// Data structure to represent the regular file [`Endpoint`].
#[derive(Debug, Clone)]
pub struct FileEndpoint {
    path: PathBuf,
    notifications: Vec<String>,
}

impl FileEndpoint {
    /// Create a new `FileEndpoint`.
    pub fn new(path: &str, notifications: &[String]) -> Self {
        let path = PathBuf::from(path);
        let notifications = notifications.into();
        Self { path, notifications }
    }
    /// Return the file path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Return all associated notification names.
    pub fn notifications(&self) -> &[String] {
        &self.notifications
    }
}

impl TryFrom<&FileConfigFile> for FileEndpoint {
    type Error = Error;

    fn try_from(value: &FileConfigFile) -> Result<Self, Self::Error> {
        if value.path.is_empty() {
            return Err(Error::InvalidEndpointConfiguration("File configuration path is blank".to_string()));
        }

        if value.notifications.is_empty() {
            return Err(Error::InvalidEndpointConfiguration(
                "File configuration has no notifications setup".to_string(),
            ));
        }

        Ok(FileEndpoint::new(value.path.as_str(), &value.notifications))
    }
}

#[typetag::deserialize(name = "file")]
impl EndpointConfig for FileConfigFile {
    fn to_endpoint(&self) -> Result<Box<dyn Endpoint + Send>, Error> {
        Ok(Box::new(FileEndpoint::try_from(self)?))
    }
}

#[async_trait]
impl Endpoint for FileEndpoint {
    async fn notify(
        &self,
        endpoint_rx: broadcast::Receiver<ValidatedNotification>,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        let path = self.path().clone();
        info!(target: LIB_LOG_TARGET, "Setting up Endpoint: File -> {}", path.to_str().unwrap_or_default());
        tokio::spawn(async move { write_file(path, endpoint_rx, shutdown).await });
        Ok(())
    }

    fn generate_keys(&self, hash_key: &Key) -> HashMap<String, HashSet<Key>> {
        let keys: HashSet<Key> = self
            .notifications()
            .iter()
            .map(|notification_name| Key::generate(notification_name.as_str(), hash_key))
            .collect();

        let mut map = HashMap::new();
        map.insert("".to_string(), keys);
        map
    }
}

async fn write_file<P: AsRef<Path>>(
    path: P,
    endpoint_rx: broadcast::Receiver<ValidatedNotification>,
    shutdown: watch::Receiver<bool>,
) -> Result<(), Error> {
    let mut rx = endpoint_rx.resubscribe();
    let mut shutdown_rx = shutdown.clone();

    let file = OpenOptions::new().read(true).append(true).create(true).open(path.as_ref()).await?;
    let mut file = BufWriter::new(file);
    loop {
        tokio::select! {
            received = rx.recv() => {
                if let Ok(message) = received {
                    let line = [message.message().text().as_bytes(), LINE_FEED].concat();
                    match file.write(line.as_slice()).await {
                        Ok(_) => (),
                        Err(e) => warn!(target: LIB_LOG_TARGET, "{}", e)
                    }

                    match file.flush().await {
                        Ok(_) => (),
                        Err(e) => warn!(target: LIB_LOG_TARGET, "{}", e),
                    };
                }
            }

            _ = shutdown_rx.changed() => {
                break;
            }
        }
    }

    file.shutdown().await?;
    Ok(())
}
