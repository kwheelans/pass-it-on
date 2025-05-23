//! Pipe [`Interface`] and [`InterfaceConfig`]  implementation
//!
//! # Server Configuration Example
//! ```toml
//! [[server.interface]]
//! type = "pipe"
//! path = '/path/to/pipe.fifo'
//! group_read_permission = true
//! ```
//!
//! # Client Configuration Example
//! ```toml
//! [[client.interface]]
//! type = "pipe"
//! path = '/path/to/pipe.fifo'
//! group_read_permission = true
//! group_write_permission = true
//! ```

#[cfg(feature = "pipe-client")]
pub(crate) mod pipe_client;
#[cfg(feature = "pipe-server")]
pub(crate) mod pipe_server;

use crate::interfaces::{Interface, InterfaceConfig};
use crate::notifications::Notification;
use crate::Error;
use async_trait::async_trait;
#[cfg(feature = "pipe-server")]
use nix::sys::stat::Mode;
use serde::Deserialize;
#[cfg(feature = "pipe-server")]
use std::path::Path;
use std::path::PathBuf;
use nix::fcntl::AT_FDCWD;
use tokio::sync::mpsc::Sender;
use tokio::sync::{broadcast, watch};

/// Data structure to represent the Named Pipe [`Interface`].
#[derive(Debug, Clone)]
pub struct PipeInterface {
    path: PathBuf,
    group_read: bool,
    group_write: bool,
    other_read: bool,
    other_write: bool,
}

/// Data structure to represent the Named Pipe [`InterfaceConfig`].
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub(crate) struct PipeConfigFile {
    path: String,
    group_read_permission: Option<bool>,
    group_write_permission: Option<bool>,
    other_read_permission: Option<bool>,
    other_write_permission: Option<bool>,
}

impl PipeInterface {
    /// Create a new `PipeInterface`.
    pub fn new(path: &str, group_read: bool, group_write: bool, other_read: bool, other_write: bool) -> Self {
        let path = PathBuf::from(path);
        Self { path, group_read, group_write, other_read, other_write }
    }

    /// Return the pipe file path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Group read permission set.
    pub fn group_read(&self) -> bool {
        self.group_read
    }

    /// Group write permission set.
    pub fn group_write(&self) -> bool {
        self.group_write
    }

    /// Other read permission set.
    pub fn other_read(&self) -> bool {
        self.other_read
    }

    /// Other write permission set.
    pub fn other_write(&self) -> bool {
        self.other_write
    }
}

impl TryFrom<&PipeConfigFile> for PipeInterface {
    type Error = Error;

    fn try_from(value: &PipeConfigFile) -> Result<Self, Self::Error> {
        if value.path.is_empty() {
            return Err(Error::InvalidInterfaceConfiguration("Pipe path is empty".to_string()));
        }

        Ok(Self {
            path: PathBuf::from(value.path.as_str()),
            group_read: value.group_read_permission.unwrap_or(false),
            group_write: value.group_write_permission.unwrap_or(false),
            other_read: value.other_read_permission.unwrap_or(false),
            other_write: value.other_write_permission.unwrap_or(false),
        })
    }
}

#[typetag::deserialize(name = "pipe")]
impl InterfaceConfig for PipeConfigFile {
    fn to_interface(&self) -> Result<Box<dyn Interface + Send>, Error> {
        Ok(Box::new(PipeInterface::try_from(self)?))
    }
}

#[async_trait]
impl Interface for PipeInterface {
    #[cfg(feature = "pipe-server")]
    async fn receive(&self, interface_tx: Sender<String>, shutdown: watch::Receiver<bool>) -> Result<(), Error> {
        use crate::interfaces::pipe::pipe_server::read_pipe;
        use tracing::info;

        const USER_RWX: Mode = Mode::S_IRWXU;
        const GROUP_READ: Mode = Mode::S_IRGRP;
        const GROUP_WRITE: Mode = Mode::S_IWGRP;
        const OTHER_READ: Mode = Mode::S_IROTH;
        const OTHER_WRITE: Mode = Mode::S_IWOTH;

        let path = self.path().clone();
        let pipe_permissions = {
            let mut permissions = vec![USER_RWX];
            if self.group_read() {
                permissions.push(GROUP_READ);
            }

            if self.group_write() {
                permissions.push(GROUP_WRITE);
            }

            if self.other_read() {
                permissions.push(OTHER_READ);
            }

            if self.other_write() {
                permissions.push(OTHER_WRITE);
            }

            create_permissions(permissions)
        };

        tokio::spawn(async move {
            if !path.exists() {
                create_pipe(&path, pipe_permissions)?
            }
            info!("Setting up Interface: Pipe on -> {}", &path.to_str().unwrap_or_default());
            read_pipe(&path, interface_tx, shutdown).await
        });
        Ok(())
    }

    #[cfg(not(feature = "pipe-server"))]
    async fn receive(&self, _interface_tx: Sender<String>, _shutdown: watch::Receiver<bool>) -> Result<(), Error> {
        Err(Error::DisabledInterfaceFeature("pipe-server".to_string()))
    }

    #[cfg(feature = "pipe-client")]
    async fn send(
        &self,
        interface_tx: broadcast::Receiver<Notification>,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        use crate::interfaces::pipe::pipe_client::write_pipe;
        use tracing::error;

        let path = self.path.clone();
        tokio::spawn(async move {
            match write_pipe(path, interface_tx, shutdown).await {
                Ok(_) => (),
                Err(error) => error!("Pipe write error {}", error),
            }
        });
        Ok(())
    }

    #[cfg(not(feature = "pipe-client"))]
    async fn send(
        &self,
        _interface_rx: broadcast::Receiver<Notification>,
        _shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        Err(Error::DisabledInterfaceFeature("pipe-client".to_string()))
    }
}

#[cfg(feature = "pipe-server")]
fn create_pipe<P: AsRef<Path>>(path: P, permissions: Mode) -> Result<(), Error> {
    match nix::unistd::mkfifo(path.as_ref(), permissions) {
        Err(e) => Err(Error::NixErrorNoError(e)),
        Ok(_) => set_permissions(path, permissions),
    }
}

#[cfg(feature = "pipe-server")]
fn create_permissions(permissions: Vec<Mode>) -> Mode {
    let mut set_permission = Mode::empty();
    for permission in permissions {
        set_permission.insert(permission)
    }
    if set_permission.is_empty() {
        set_permission.insert(Mode::S_IRWXU)
    }
    set_permission
}

#[cfg(feature = "pipe-server")]
fn set_permissions<P: AsRef<Path>>(path: P, permissions: Mode) -> Result<(), Error> {
    use nix::sys::stat::FchmodatFlags;
    nix::sys::stat::fchmodat(AT_FDCWD, path.as_ref(), permissions, FchmodatFlags::NoFollowSymlink)?;
    Ok(())
}

#[cfg(feature = "pipe-server")]
async fn cleanup_pipe<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    std::fs::remove_file(path)?;
    Ok(())
}
