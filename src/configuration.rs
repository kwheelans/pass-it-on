use crate::endpoints::{Endpoint, EndpointChannel};
use crate::interfaces::Interface;
use crate::notifications::{Key, ValidatedNotification};
use crate::Error;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

#[cfg(feature = "client")]
pub mod client_configuration_file;
#[cfg(feature = "server")]
pub mod server_configuration_file;

#[cfg(feature = "client")]
pub use self::client_configuration_file::ClientConfigFileParser;
#[cfg(feature = "server")]
pub use self::server_configuration_file::ServerConfigFileParser;

#[cfg(feature = "server")]
/// Server configuration that can be used to start the server.
#[derive(Debug)]
pub struct ServerConfiguration {
    key: Key,
    interfaces: Vec<Box<dyn Interface + Send>>,
    endpoints: Vec<Box<dyn Endpoint + Send>>,
}

#[cfg(feature = "server")]
impl ServerConfiguration {
    /// Create a new `ServerConfiguration`.
    pub fn new(
        key: Key,
        interfaces: Vec<Box<dyn Interface + Send>>,
        endpoints: Vec<Box<dyn Endpoint + Send>>,
    ) -> Result<Self, Error> {
        let config = Self { key, interfaces, endpoints };
        Self::validate(config)
    }

    pub(crate) fn endpoint_channels(&self) -> Vec<EndpointChannel> {
        let mut endpoints = Vec::new();
        for endpoint in &self.endpoints {
            let (endpoint_tx, _endpoint_rx): (Sender<ValidatedNotification>, Receiver<ValidatedNotification>) =
                broadcast::channel(100);
            let keys = endpoint.generate_keys(&self.key);
            endpoints.push(EndpointChannel::from(endpoint.clone(), endpoint_tx, keys));
        }
        endpoints
    }

    /// Return server key value as a byte array.
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// Return all server interfaces.
    pub fn interfaces(&self) -> Vec<Box<dyn Interface + Send>> {
        self.interfaces.clone()
    }

    /// Return all server endpoints.
    pub fn endpoints(&self) -> &[Box<dyn Endpoint + Send>] {
        &self.endpoints
    }

    fn validate(config: ServerConfiguration) -> Result<ServerConfiguration, Error> {
        if config.interfaces.is_empty() {
            return Err(Error::MissingInterface);
        }

        if config.endpoints.is_empty() {
            return Err(Error::MissingEndpoint);
        }

        Ok(config)
    }
}

/// Client configuration that can be used to start the client.
#[derive(Debug)]
pub struct ClientConfiguration {
    key: Key,
    interfaces: Vec<Box<dyn Interface + Send>>,
}

impl ClientConfiguration {
    /// Create a new `ClientConfiguration`.
    pub fn new(key: Key, interfaces: Vec<Box<dyn Interface + Send>>) -> Result<Self, Error> {
        let config = Self { key, interfaces };
        Self::validate(config)
    }
    /// Return client key value.
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// Return all client interfaces.
    pub fn interfaces(&self) -> Vec<Box<dyn Interface + Send>> {
        self.interfaces.clone()
    }

    fn validate(config: ClientConfiguration) -> Result<ClientConfiguration, Error> {
        if config.interfaces.is_empty() {
            return Err(Error::MissingInterface);
        }
        Ok(config)
    }
}

fn valid_key_length(key: &str) -> Result<(), Error> {
    match key.len() == 32 {
        true => Ok(()),
        false => Err(Error::InvalidKeyLength(key.len() as u8)),
    }
}
