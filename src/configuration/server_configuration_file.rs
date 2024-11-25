use crate::configuration::{collect_endpoints, collect_interfaces, ServerConfiguration};
use crate::endpoints::{Endpoint, EndpointConfig};
use crate::interfaces::{Interface, InterfaceConfig};
use crate::Error;
use serde::Deserialize;

/// Server configuration parsed from TOML that handles any [`InterfaceConfig`][`crate::interfaces::InterfaceConfig`]
/// and [`EndpointConfig`][`crate::endpoints::EndpointConfig`].
#[derive(Deserialize, Debug)]
pub(super) struct ServerConfigFileParser {
    server: ServerConfigFile,
}

/// Serde compatible representation of [`ServerConfiguration`]
#[derive(Deserialize, Debug)]
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

impl TryFrom<ServerConfigFile> for ServerConfiguration {
    type Error = Error;

    fn try_from(value: ServerConfigFile) -> Result<Self, Self::Error> {
        let interfaces: Vec<Box<dyn Interface + Send>> = collect_interfaces(value.interface)?;
        let endpoints: Vec<Box<dyn Endpoint + Send>> = collect_endpoints(value.endpoint)?;

        ServerConfiguration::new(value.key.as_str(), interfaces, endpoints)
    }
}
