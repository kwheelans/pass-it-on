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
//!     let config = ClientConfigFileParser::from(CLIENT_TOML_CONFIG)?;
//!     let notification_key = Key::generate("NOTIFICATION_NAME", &config.key());
//!     let (interface_tx, interface_rx) = mpsc::channel(100);
//!
//!     let messages = vec![
//!         Notification::new("A message to be sent", &notification_key),
//!         Notification::new("Another message", &notification_key),
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
//! | Feature      | Description                                                                                                                 |
//! |--------------|-----------------------------------------------------------------------------------------------------------------------------|
//! | client       | Enables the client but not any particular interface.                                                                        |
//! | endpoints    | Enables the Endpoint and EndpointConfig traits.                                                                             |
//! | file         | Enables the regular file endpoint.                                                                                          |
//! | http         | Enables the HTTP interface client and server.                                                                               |
//! | http-client  | Enables the HTTP interface for just client.                                                                                 |
//! | http-server  | Enables the HTTP interface for just server.                                                                                 |
//! | interfaces   | Enables the Interface and InterfaceConfig traits.                                                                           |
//! | matrix       | Enables the matrix endpoint.                                                                                                |
//! | pipe         | Enables the named pipe interface client and server. **(Unix only)**                                                         |
//! | pipe-client  | Enables the named pipe interface client. **(Unix only)**                                                                    |
//! | pipe-server  | Enables the named pipe interface server. **(Unix only)**                                                                    |
//! | server       | Enables the server but not any particular interface or endpoint.                                                            |
//! | server-bin   | Enables the building of the provided `pass-it-on-server` server binary while not require any specific interface or endpoint |
//! | vendored-tls | Enables vendored tls for reqwest.                                                                                           |

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
pub use self::client::start_client;
#[cfg(feature = "client")]
pub use self::configuration::{ClientConfigFileParser, ClientConfiguration};
#[cfg(feature = "server")]
pub use self::configuration::{ServerConfigFileParser, ServerConfiguration};
pub use self::error::Error;
#[cfg(feature = "server")]
pub use self::server::start_server;

/// Logging target value used for the library.
pub const LIB_LOG_TARGET: &str = "pass_it_on";
