#![warn(missing_docs)]
//! # Pass-It-On
//! A library that provides simple notification client and server that receives messages and passes them on to endpoints
//!
//!
//! ## Client Example
//! To use the client to pass messages to the server you will need to pass a valid [`ClientConfiguration`]
//! and a channel receiver to the start client function.  It will monitor that channel for incoming
//! [`Notification`][crate::notifications::Notification] values and send them in the expected format to server.
//!
//! ```
//! # use pass_it_on::notifications::{Key, Notification};
//! # use pass_it_on::{ClientConfigFileParser, start_client, Error};
//! # use tokio::sync::mpsc;
//! #
//! # const CLIENT_TOML_CONFIG: &str = r#"
//! #    [client]
//! #    key = "UVXu7wtbXHWNgAr6rWyPnaZbZK9aYin8"
//! #
//! #    [[client.interface]]
//! #    type = "http"
//! #    port = 8080
//! #
//! # "#;
//!
//! # #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     const NOTIFICATION_NAME: &str = "test1";
//!     let config = ClientConfiguration::from_toml(CLIENT_TOML_CONFIG)?;
//!     let (interface_tx, interface_rx) = mpsc::channel(100);
//!
//!     let messages = vec![
//!         Message::new("A message to be sent").to_client_ready_message(NOTIFICATION_NAME),
//!         Message::new("Another message").to_client_ready_message(NOTIFICATION_NAME),
//!     ];
//!
//!     for message in messages {
//!         if let Err(send_error) = interface_tx.send(message).await {
//!             println!("Send Error: {}", send_error);
//!         }
//!     }
//!
//!     start_client(config, interface_rx, None).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! | Feature                 | Description                                                                                                            |
//! |-------------------------|------------------------------------------------------------------------------------------------------------------------|
//! | client                  | Enables the client but not any particular interface.                                                                   |
//! | discord                 | Enables the discord webhook endpoint.                                                                                  |
//! | email                   | Enables the email endpoint.                                                                                            |
//! | endpoints               | Enables the Endpoint and EndpointConfig traits.                                                                        |
//! | file                    | Enables the regular file endpoint.                                                                                     |
//! | http                    | Enables the HTTP interface client and server.                                                                          |
//! | http-client             | Enables the HTTP interface for just client.                                                                            |
//! | http-server             | Enables the HTTP interface for just server.                                                                            |
//! | interfaces              | Enables the Interface and InterfaceConfig traits.                                                                      |
//! | matrix                  | Enables the matrix endpoint.                                                                                           |
//! | parse-cfg               | Enables parsing of client or server configurations from TOML when those features are also enabled.                     |
//! | pipe                    | Enables the named pipe interface client and server. **(Unix only)**                                                    |
//! | pipe-client             | Enables the named pipe interface client. **(Unix only)**                                                               |
//! | pipe-server             | Enables the named pipe interface server. **(Unix only)**                                                               |
//! | server                  | Enables the server but not any particular interface or endpoint.                                                       |
//! | server-bin-full         | Enables the building of the provided `pass-it-on-server` binary with all available interfaces and endpoints            |
//! | server-bin-minimal      | Enables the building of the provided `pass-it-on-server` binary while not requiring any specific interface or endpoint |
//! | rustls-tls-native-roots | Enables rustls-tls-native-roots for reqwest.                                                                           |

#[cfg(feature = "client")]
mod client;
#[cfg(any(feature = "server", feature = "client"))]
mod configuration;
#[cfg(feature = "endpoints")]
pub mod endpoints;
mod error;
#[cfg(feature = "interfaces")]
pub mod interfaces;
pub mod notifications;
#[cfg(feature = "server")]
mod server;
#[cfg(any(feature = "server", feature = "client"))]
pub(crate) mod shutdown;

#[cfg(feature = "client")]
pub use self::client::{start_client, start_client_arc};
#[cfg(all(feature = "client", feature = "parse-cfg"))]
pub use self::configuration::client_configuration_file::ClientConfigFile;
#[cfg(all(feature = "server", feature = "parse-cfg"))]
pub use self::configuration::server_configuration_file::ServerConfigFile;
#[cfg(feature = "client")]
pub use self::configuration::ClientConfiguration;
#[cfg(feature = "server")]
pub use self::configuration::ServerConfiguration;
pub use self::error::Error;
#[cfg(feature = "server")]
pub use self::server::start_server;
#[cfg(all(feature = "server", feature = "matrix"))]
pub use self::server::verify_matrix_devices;

#[allow(dead_code)]
const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
#[allow(dead_code)]
const CHANNEL_BUFFER: usize = 200;
const KEY_CONTEXT: &str = "pass-it-on 2024-02-18 client-server shared-key";
