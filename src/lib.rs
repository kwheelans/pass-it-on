#![warn(missing_docs)]
//! # Pass-It-On
//! A library that provides simple notification client and server that receives messages and passes them on to endpoints

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
