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
//! use pass_it_on::notifications::{Key, Notification};
//! use pass_it_on::{ClientConfigFileParser, start_client, Error};
//! use tokio::sync::mpsc;
//!
//! const CLIENT_TOML_CONFIG: &str = r#"
//!     [client]
//!     key = "UVXu7wtbXHWNgAr6rWyPnaZbZK9aYin8"
//!
//!     [[client.interface]]
//!     type = "http"
//!     port = 8080
//!
//! "#;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     let config = ClientConfigFileParser::from(CLIENT_TOML_CONFIG)?;
//!     let notification_key = Key::generate("NOTIFICATION_NAME", &config.key());
//!     let (interface_tx, interface_rx) = mpsc::channel(100);
//!
//!     let messages = vec![
//!         Notification::new("A message to be sent", notification_key.as_bytes()),
//!         Notification::new("Another message", notification_key.as_bytes()),
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
