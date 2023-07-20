//! Endpoints for the server

use crate::notifications::{Key, ValidatedNotification};
use crate::Error;
use async_trait::async_trait;
use dyn_clone::DynClone;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use tokio::sync::{broadcast, watch};

#[cfg(feature = "discord")]
pub mod discord;
#[cfg(feature = "file")]
pub mod file;
#[cfg(feature = "matrix")]
pub mod matrix;

/// A data structure that can be deserialized and converted into an [`Endpoint`].
#[typetag::deserialize(tag = "type")]
pub trait EndpointConfig {
    /// Convert this `EndpointConfig` into an [`Endpoint`].
    fn to_endpoint(&self) -> Result<Box<dyn Endpoint + Send>, Error>;
}

/// A data structure that contains information and functions need the server needs to send messages to endpoint.
#[async_trait]
pub trait Endpoint: DynClone + Send + Debug {
    /// Implements the server sending notifications to the `Endpoint`.
    async fn notify(
        &self,
        endpoint_rx: broadcast::Receiver<ValidatedNotification>,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error>;

    /// Generates a [`HashMap`] where the keys represent a sub-group of notifications.
    ///
    /// This is useful for endpoints like Matrix where multiple rooms can be setup, but all notifications going to the endpoint
    /// do not go to all rooms.
    /// For endpoints like File where this is not applicable all notifications can go under a single key.
    fn generate_keys(&self, hash_key: &Key) -> HashMap<String, HashSet<Key>>;
}

dyn_clone::clone_trait_object!(Endpoint);

#[derive(Clone)]
pub(crate) struct EndpointChannel {
    endpoint: Box<dyn Endpoint + Send>,
    channel: broadcast::Sender<ValidatedNotification>,
    keys: HashMap<String, HashSet<Key>>,
}

impl EndpointChannel {
    pub fn from(
        endpoint: Box<dyn Endpoint + Send>,
        channel: broadcast::Sender<ValidatedNotification>,
        keys: HashMap<String, HashSet<Key>>,
    ) -> Self {
        EndpointChannel { endpoint, channel, keys }
    }

    pub fn endpoint(&self) -> &Box<dyn Endpoint + Send> {
        &self.endpoint
    }

    pub fn channel_receiver(&self) -> broadcast::Receiver<ValidatedNotification> {
        self.channel.subscribe()
    }

    pub fn channel_sender(&self) -> broadcast::Sender<ValidatedNotification> {
        self.channel.clone()
    }

    pub fn keys(&self) -> &HashMap<String, HashSet<Key>> {
        &self.keys
    }
}

pub(crate) async fn setup_endpoints(
    endpoints: Vec<EndpointChannel>,
    shutdown: watch::Receiver<bool>,
) -> Result<(), Error> {
    for channel in endpoints {
        channel.endpoint().notify(channel.channel_receiver(), shutdown.clone()).await?
    }
    Ok(())
}
