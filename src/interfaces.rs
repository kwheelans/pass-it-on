//! Interfaces for the server and client

use crate::notifications::Notification;
use crate::Error;
use async_trait::async_trait;
use dyn_clone::DynClone;
use std::fmt::Debug;
use tokio::sync::{broadcast, mpsc, watch};

#[cfg(all(unix, feature = "pipe"))]
pub mod pipe;

#[cfg(any(feature = "http-client", feature = "http-server"))]
pub mod http;

/// A data structure that can be deserialized and converted into an [`Interface`].
#[typetag::deserialize(tag = "type")]
pub trait InterfaceConfig {
    /// Convert this `InterfaceConfig` into an [`Interface`].
    fn to_interface(&self) -> Box<dyn Interface + Send>;

    /// Perform any necessary validations on the configuration to ensure it's usable.
    fn validate(&self) -> Result<(), Error>;
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
