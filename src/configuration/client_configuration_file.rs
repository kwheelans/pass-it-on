use crate::configuration::{valid_key_length, ClientConfiguration};
use crate::interfaces::{Interface, InterfaceConfig};
use crate::Error;
use serde::Deserialize;

/// Client configuration parsed from TOML that handles any [`InterfaceConfig`][`crate::interfaces::InterfaceConfig`].
#[derive(Deserialize)]
pub struct ClientConfigFileParser {
    client: ClientConfigFile,
}

#[derive(Deserialize)]
struct ClientConfigFile {
    key: String,
    interface: Vec<Box<dyn InterfaceConfig>>,
}

impl ClientConfigFileParser {
    /// Parse [`ClientConfiguration`] from provided TOML.
    pub fn from(string: &str) -> Result<ClientConfiguration, Error> {
        let parsed: ClientConfigFileParser = toml::from_str(string)?;
        parsed.validate()
    }

    fn validate(&self) -> Result<ClientConfiguration, Error> {
        self.client.validate()
    }
}

impl ClientConfigFile {
    fn key(&self) -> [u8; 32] {
        self.key.clone().into_bytes().try_into().unwrap()
    }

    fn validate(&self) -> Result<ClientConfiguration, Error> {
        valid_key_length(self.key.as_str())?;

        for cfg in self.interface.iter() {
            cfg.validate()?
        }

        let interfaces: Vec<Box<dyn Interface + Send>> = self.interface.iter().map(|cfg| cfg.to_interface()).collect();

        ClientConfiguration::new(self.key(), interfaces)
    }
}
