use crate::configuration::{collect_interfaces, valid_key_length, ClientConfiguration};
use crate::interfaces::{Interface, InterfaceConfig};
use crate::notifications::Key;
use crate::Error;
use serde::Deserialize;

/// Client configuration parsed from TOML that handles any [`InterfaceConfig`][`crate::interfaces::InterfaceConfig`].
#[derive(Deserialize)]
pub(super) struct ClientConfigFileParser {
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
        parsed.client.try_into()
    }
}

impl ClientConfigFile {
    fn key(&self) -> [u8; 32] {
        self.key.clone().into_bytes().try_into().unwrap()
    }
}

impl TryFrom<ClientConfigFile> for ClientConfiguration {
    type Error = Error;

    fn try_from(value: ClientConfigFile) -> Result<Self, Self::Error> {
        valid_key_length(value.key.as_str())?;
        let key = value.key();
        let interfaces: Vec<Box<dyn Interface + Send>> = collect_interfaces(value.interface)?;

        ClientConfiguration::new(Key::from_bytes(&key), interfaces)
    }
}
