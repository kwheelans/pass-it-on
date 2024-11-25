use crate::configuration::{collect_interfaces, ClientConfiguration};
use crate::interfaces::{Interface, InterfaceConfig};
use crate::Error;
use serde::Deserialize;

/// Client configuration parsed from TOML that handles any [`InterfaceConfig`][`crate::interfaces::InterfaceConfig`].
#[derive(Deserialize, Debug)]
pub(super) struct ClientConfigFileParser {
    client: ClientConfigFile,
}

/// Serde compatible representation of [`ClientConfiguration`]
#[derive(Deserialize, Debug)]
pub struct ClientConfigFile {
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

impl TryFrom<ClientConfigFile> for ClientConfiguration {
    type Error = Error;

    fn try_from(value: ClientConfigFile) -> Result<Self, Self::Error> {
        let interfaces: Vec<Box<dyn Interface + Send>> = collect_interfaces(value.interface)?;
        ClientConfiguration::new(value.key.as_str(), interfaces)
    }
}
