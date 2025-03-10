//! Interfaces for the server and client

use crate::notifications::Notification;
use crate::Error;
use async_trait::async_trait;
use dyn_clone::DynClone;
use std::fmt::Debug;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, watch};

#[cfg(all(unix, any(feature = "pipe-client", feature = "pipe-server", feature = "pipe")))]
pub mod pipe;

#[cfg(any(feature = "http-client", feature = "http-server"))]
pub mod http;

#[allow(dead_code)]
pub(crate) const SECOND: Duration = Duration::from_secs(1);
#[allow(dead_code)]
pub(crate) const NANOSECOND: Duration = Duration::from_nanos(1);

/// A data structure that can be deserialized and converted into an [`Interface`].
#[typetag::deserialize(tag = "type")]
pub trait InterfaceConfig: Debug {
    /// Convert this `InterfaceConfig` into an [`Interface`].
    fn to_interface(&self) -> Result<Box<dyn Interface + Send>, Error>;
}

/// A data structure that contains information and functions needed to communicate on a particular interface between the server and client.
#[async_trait]
pub trait Interface: DynClone + Send + Debug {
    /// Implements the server receiving notifications from the `Interface`.
    async fn receive(&self, interface_tx: mpsc::Sender<String>, shutdown: watch::Receiver<bool>) -> Result<(), Error>;

    /// Implements the client sending notifications to the `Interface`.
    async fn send(
        &self,
        interface_rx: broadcast::Receiver<Notification>,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error>;
}

dyn_clone::clone_trait_object!(Interface);

#[cfg(feature = "server")]
pub(crate) async fn setup_server_interfaces(
    interfaces: Vec<Box<dyn Interface + Send>>,
    interface_tx: mpsc::Sender<String>,
    shutdown: watch::Receiver<bool>,
) -> Result<(), Error> {
    for interface in interfaces {
        interface.receive(interface_tx.clone(), shutdown.clone()).await?;
    }
    Ok(())
}

#[cfg(feature = "client")]
pub(crate) async fn setup_client_interfaces(
    interfaces: Vec<Box<dyn Interface + Send>>,
    interface_rx: broadcast::Receiver<Notification>,
    shutdown: watch::Receiver<bool>,
) -> Result<(), Error> {
    for interface in interfaces {
        interface.send(interface_rx.resubscribe(), shutdown.clone()).await?
    }
    Ok(())
}
