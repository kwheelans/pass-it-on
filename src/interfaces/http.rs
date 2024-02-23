//! HTTP [`Interface`] and [`InterfaceConfig`] implementation
//!
//! # Server Configuration Example
//! ## Configuration for Localhost
//! ```toml
//! [[server.interface]]
//! type = "http"
//! host = "http://localhost"
//! port = 8080
//! ```
//!
//! ## Configuration with TLS
//! ```toml
//! [[server.interface]]
//! type = "http"
//! host = "example.com"
//! port = 8080
//! tls = true
//! tls_cert_path = "/path/to/certificate/cert.pem"
//! tls_key_path = "/path/to/private/key/key.pem"
//! ```
//!
//! # Client Configuration Example
//! ```toml
//! [[client.interface]]
//! type = "http"
//! host = "127.0.0.1"
//! port = 8080
//! ```

#[cfg(feature = "http-client")]
pub(crate) mod http_client;
#[cfg(feature = "http-server")]
pub(crate) mod http_server;

use crate::interfaces::{Interface, InterfaceConfig};
use crate::notifications::Notification;
use crate::Error;
use async_trait::async_trait;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::sync::{broadcast, mpsc, watch};
use url::{ParseError, Url};

const LOCALHOST: &str = "http://localhost";
const HTTP: &str = "http";
const HTTPS: &str = "https";
const DEFAULT_PORT: u16 = 8080;

/// Data structure to represent the HTTP Socket [`Interface`].
#[derive(Debug, Clone)]
pub struct HttpSocketInterface {
    host: Url,
    tls: bool,
    port: u16,
    tls_cert_path: Option<PathBuf>,
    tls_key_path: Option<PathBuf>,
}

/// Data structure to represent the HTTP Socket [`InterfaceConfig`].
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(default)]
pub(crate) struct HttpSocketConfigFile {
    pub host: String,
    pub tls: bool,
    pub port: i64,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

impl HttpSocketInterface {
    /// Create a new `HttpSocketInterface`.
    pub fn new<P: AsRef<str>>(host_url: &Url, cert_path: Option<P>, key_path: Option<P>) -> Self {
        let host = host_url.clone();
        let tls = host.scheme().eq_ignore_ascii_case(HTTPS);
        let port = host.port().unwrap_or(DEFAULT_PORT);
        let tls_cert_path = cert_path.map(|p| PathBuf::from(p.as_ref()));
        let tls_key_path = key_path.map(|p| PathBuf::from(p.as_ref()));
        Self { host, tls, port, tls_cert_path, tls_key_path }
    }

    /// Return the IP address.
    pub fn host(&self) -> &str {
        self.host.as_str()
    }

    /// Return the IP address if it exists or the default address(127.0.0.1).
    pub fn sockets(&self) -> Result<Vec<SocketAddr>, Error> {
        Ok(self.host.socket_addrs(|| Some(self.port()))?)
    }

    /// Return the port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Return if interface should use TLS
    pub fn tls(&self) -> bool {
        self.tls
    }

    /// Return path the TLS certificate
    pub fn tls_cert_path(&self) -> &Option<PathBuf> {
        &self.tls_cert_path
    }

    /// Return path the TLS private key
    pub fn tls_key_path(&self) -> &Option<PathBuf> {
        &self.tls_key_path
    }
}

impl Default for HttpSocketConfigFile {
    fn default() -> Self {
        Self { host: LOCALHOST.into(), tls: false, port: DEFAULT_PORT as i64, tls_cert_path: None, tls_key_path: None }
    }
}

impl Default for HttpSocketInterface {
    fn default() -> Self {
        Self {
            host: Url::parse(LOCALHOST).unwrap(),
            tls: false,
            port: DEFAULT_PORT,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

impl TryFrom<&HttpSocketConfigFile> for HttpSocketInterface {
    type Error = Error;

    fn try_from(value: &HttpSocketConfigFile) -> Result<Self, Self::Error> {
        if !(value.port < u16::MAX as i64 && value.port > u16::MIN as i64) {
            return Err(Error::InvalidPortNumber(value.port));
        }
        let mut url = parse_url(value.host.as_str())?;
        match value.tls {
            true => url.set_scheme(HTTPS),
            false => url.set_scheme(HTTP),
        }
        .unwrap();

        url.set_port(Some(value.port as u16)).unwrap();
        Ok(HttpSocketInterface::new(&url, value.tls_cert_path.as_ref(), value.tls_key_path.as_ref()))
    }
}

#[typetag::deserialize(name = "http")]
impl InterfaceConfig for HttpSocketConfigFile {
    fn to_interface(&self) -> Result<Box<dyn Interface + Send>, Error> {
        Ok(Box::new(HttpSocketInterface::try_from(self)?))
    }
}

#[async_trait]
impl Interface for HttpSocketInterface {
    #[cfg(feature = "http-server")]
    async fn receive(&self, interface_tx: mpsc::Sender<String>, shutdown: watch::Receiver<bool>) -> Result<(), Error> {
        use crate::interfaces::http::http_server::start_monitoring;

        if self.tls && (self.tls_cert_path().is_none() || self.tls_cert_path().is_none()) {
            Err(Error::InvalidInterfaceConfiguration(
                "Both tls_cert_path and tls_cert_path must be provided for a TLS server".into(),
            ))
        } else {
            for socket in self.sockets()? {
                let tls = self.tls;
                let itx = interface_tx.clone();
                let srx = shutdown.clone();
                let cert_path = self.tls_cert_path.clone();
                let key_path = self.tls_key_path.clone();
                tokio::spawn(async move { start_monitoring(itx, srx, socket, tls, cert_path, key_path).await });
            }
            Ok(())
        }
    }

    #[cfg(not(feature = "http-server"))]
    async fn receive(
        &self,
        _interface_tx: mpsc::Sender<String>,
        _shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        Err(Error::DisabledInterfaceFeature("http-server".to_string()))
    }

    #[cfg(feature = "http-client")]
    async fn send(
        &self,
        interface_rx: broadcast::Receiver<Notification>,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        use crate::interfaces::http::http_client::start_sending;

        let mut url = self.host.clone();
        url.set_path("notification");

        tokio::spawn(async move { start_sending(interface_rx, shutdown, url.as_str()).await });
        Ok(())
    }

    #[cfg(not(feature = "http-client"))]
    async fn send(
        &self,
        _interface_rx: broadcast::Receiver<Notification>,
        _shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        Err(Error::DisabledInterfaceFeature("http-client".to_string()))
    }
}

fn parse_url(value: &str) -> Result<Url, Error> {
    match Url::parse(value) {
        Ok(url) => Ok(url),
        Err(error) if error == ParseError::RelativeUrlWithoutBase => {
            parse_url(format!("{}://{}", HTTP, value).as_str())
        }
        Err(error) => Err(error.into()),
    }
}
