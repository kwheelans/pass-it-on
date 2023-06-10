use crate::configuration::{valid_key_length, ServerConfiguration};
use crate::endpoints::{Endpoint, EndpointConfig};
use crate::interfaces::{Interface, InterfaceConfig};
use crate::Error;
use serde::Deserialize;

/// Server configuration parsed from TOML that handles any [`InterfaceConfig`][`crate::interfaces::InterfaceConfig`]
/// and [`EndpointConfig`][`crate::endpoints::EndpointConfig`].
#[derive(Deserialize)]
pub struct ServerConfigFileParser {
    server: ServerConfigFile,
}

#[derive(Deserialize)]
struct ServerConfigFile {
    key: String,
    interface: Vec<Box<dyn InterfaceConfig>>,
    endpoint: Vec<Box<dyn EndpointConfig>>,
}

impl ServerConfigFileParser {
    /// Parse [`ServerConfiguration`] from provided TOML
    pub fn from(string: &str) -> Result<ServerConfiguration, Error> {
        let parsed: ServerConfigFileParser = toml::from_str(string)?;
        parsed.validate()
    }

    fn validate(&self) -> Result<ServerConfiguration, Error> {
        self.server.validate()
    }
}

impl ServerConfigFile {
    fn key(&self) -> [u8; 32] {
        self.key.clone().into_bytes().try_into().unwrap()
    }

    fn validate(&self) -> Result<ServerConfiguration, Error> {
        valid_key_length(self.key.as_str())?;

        for cfg in self.interface.iter() {
            cfg.validate()?
        }

        for cfg in self.endpoint.iter() {
            cfg.validate()?
        }

        let interfaces: Vec<Box<dyn Interface + Send>> = self.interface.iter().map(|cfg| cfg.to_interface()).collect();
        let endpoints: Vec<Box<dyn Endpoint + Send>> = self.endpoint.iter().map(|cfg| cfg.to_endpoint()).collect();

        ServerConfiguration::new(self.key(), interfaces, endpoints)
    }
}
