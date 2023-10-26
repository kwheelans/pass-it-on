#[cfg(all(feature = "parse-cfg", feature = "client"))]
pub mod client_configuration_file;
#[cfg(all(feature = "parse-cfg", feature = "server"))]
pub mod server_configuration_file;

#[cfg(feature = "server")]
use crate::endpoints::{Endpoint, EndpointChannel, EndpointConfig};
use crate::interfaces::{Interface, InterfaceConfig};
use crate::notifications::Key;
use crate::Error;

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
        use crate::notifications::ValidatedNotification;
        use crate::CHANNEL_BUFFER;
        use tokio::sync::broadcast;
        use tokio::sync::broadcast::{Receiver, Sender};

        let mut endpoints = Vec::new();
        for endpoint in &self.endpoints {
            let (endpoint_tx, _endpoint_rx): (Sender<ValidatedNotification>, Receiver<ValidatedNotification>) =
                broadcast::channel(CHANNEL_BUFFER);
            let keys = endpoint.generate_keys(&self.key);
            endpoints.push(EndpointChannel::from(endpoint.clone(), endpoint_tx, keys));
        }
        endpoints
    }

    /// Return server [`Key`] value.
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

#[cfg(all(feature = "parse-cfg", feature = "server"))]
impl TryFrom<&str> for ServerConfiguration {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        server_configuration_file::ServerConfigFileParser::from(value)
    }
}

#[cfg(feature = "client")]
/// Client configuration that can be used to start the client.
#[derive(Debug)]
pub struct ClientConfiguration {
    key: Key,
    interfaces: Vec<Box<dyn Interface + Send>>,
}

#[cfg(feature = "client")]
impl ClientConfiguration {
    /// Create a new `ClientConfiguration`.
    pub fn new(key: Key, interfaces: Vec<Box<dyn Interface + Send>>) -> Result<Self, Error> {
        let config = Self { key, interfaces };
        Self::validate(config)
    }

    /// Return client [`Key`] value.
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

#[cfg(all(feature = "parse-cfg", feature = "client"))]
impl TryFrom<&str> for ClientConfiguration {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        client_configuration_file::ClientConfigFileParser::from(value)
    }
}

#[cfg(all(feature = "parse-cfg", any(feature = "client", feature = "server")))]
fn collect_interfaces(
    interface_configs: Vec<Box<dyn InterfaceConfig>>,
) -> Result<Vec<Box<dyn Interface + Send>>, Error> {
    interface_configs.iter().map(|cfg| cfg.to_interface()).collect()
}

#[cfg(all(feature = "parse-cfg", feature = "server"))]
fn collect_endpoints(endpoint_configs: Vec<Box<dyn EndpointConfig>>) -> Result<Vec<Box<dyn Endpoint + Send>>, Error> {
    endpoint_configs.iter().map(|cfg| cfg.to_endpoint()).collect()
}
