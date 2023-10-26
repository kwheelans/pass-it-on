use crate::configuration::{collect_endpoints, collect_interfaces, valid_key_length, ServerConfiguration};
use crate::endpoints::{Endpoint, EndpointConfig};
use crate::interfaces::{Interface, InterfaceConfig};
use crate::notifications::Key;
use crate::Error;
use serde::Deserialize;

/// Server configuration parsed from TOML that handles any [`InterfaceConfig`][`crate::interfaces::InterfaceConfig`]
/// and [`EndpointConfig`][`crate::endpoints::EndpointConfig`].
#[derive(Deserialize)]
pub(super) struct ServerConfigFileParser {
    server: ServerConfigFile,
}

/// Serde compatible representation of [`ServerConfiguration`]
#[derive(Deserialize)]
pub struct ServerConfigFile {
    key: String,
    interface: Vec<Box<dyn InterfaceConfig>>,
    endpoint: Vec<Box<dyn EndpointConfig>>,
}

impl ServerConfigFileParser {
    /// Parse [`ServerConfiguration`] from provided TOML
    pub fn from(string: &str) -> Result<ServerConfiguration, Error> {
        let parsed: ServerConfigFileParser = toml::from_str(string)?;
        parsed.server.try_into()
    }
}

impl ServerConfigFile {
    fn key(&self) -> [u8; 32] {
        self.key.clone().into_bytes().try_into().unwrap()
    }
}

impl TryFrom<ServerConfigFile> for ServerConfiguration {
    type Error = Error;

    fn try_from(value: ServerConfigFile) -> Result<Self, Self::Error> {
        valid_key_length(value.key.as_str())?;

        let key = Key::from_bytes(&value.key());
        let interfaces: Vec<Box<dyn Interface + Send>> = collect_interfaces(value.interface)?;
        let endpoints: Vec<Box<dyn Endpoint + Send>> = collect_endpoints(value.endpoint)?;

        ServerConfiguration::new(key, interfaces, endpoints)
    }
}
