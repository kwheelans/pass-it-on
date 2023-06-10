//! HTTP [`Interface`] and [`InterfaceConfig`] implementation

#[cfg(feature = "http-client")]
pub(crate) mod http_client;
#[cfg(feature = "http-server")]
pub(crate) mod http_server;

use crate::interfaces::{Interface, InterfaceConfig};
use crate::notifications::Notification;
use crate::Error;
use async_trait::async_trait;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use tokio::sync::{broadcast, mpsc, watch};

const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

/// Data structure to represent the HTTP Socket [`Interface`].
#[derive(Debug, Clone)]
pub struct HttpSocketInterface {
    ip: Option<String>,
    port: u32,
}

/// Data structure to represent the HTTP Socket [`InterfaceConfig`].
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub(crate) struct HttpSocketConfigFile {
    pub ip: Option<String>,
    pub port: i64,
}

impl HttpSocketInterface {
    /// Create a new `HttpSocketInterface`.
    pub fn new(ip: Option<&str>, port: u32) -> Self {
        let ip = ip.map(|ip| ip.to_string());
        Self { ip, port }
    }

    /// Return the IP address.
    pub fn ip(&self) -> Option<&str> {
        self.ip.as_deref()
    }

    /// Return the IP address if it exists or the default address(127.0.0.1).
    pub fn ip_or_default(&self) -> IpAddr {
        match self.ip() {
            None => IpAddr::from(LOCALHOST),
            Some(ip) => IpAddr::from_str(ip).unwrap_or(IpAddr::from(LOCALHOST)),
        }
    }

    /// Return the port.
    pub fn port(&self) -> u16 {
        self.port as u16
    }
}

#[typetag::deserialize(name = "http")]
impl InterfaceConfig for HttpSocketConfigFile {
    fn to_interface(&self) -> Box<dyn Interface + Send> {
        Box::new(HttpSocketInterface::new(self.ip.as_deref(), self.port as u32))
    }

    fn validate(&self) -> Result<(), Error> {
        match self.port < u16::MAX as i64 && self.port > u16::MIN as i64 {
            true => Ok(()),
            false => Err(Error::InvalidPortNumber(self.port)),
        }
    }
}

#[async_trait]
impl Interface for HttpSocketInterface {
    #[cfg(feature = "http-server")]
    async fn receive(&self, interface_tx: mpsc::Sender<String>, shutdown: watch::Receiver<bool>) -> Result<(), Error> {
        use crate::interfaces::http::http_server::start_monitoring;
        use std::net::SocketAddr;

        let socket = SocketAddr::new(self.ip_or_default(), self.port());

        tokio::spawn(async move { start_monitoring(interface_tx.clone(), shutdown, socket).await });
        Ok(())
    }

    #[cfg(not(feature = "http-server"))]
    async fn receive(
        &self,
        _interface_tx: mpsc::Sender<String>,
        _shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        Err(Error::DisabledInterfaceFunction("HTTP receive".to_string()))
    }

    #[cfg(feature = "http-client")]
    async fn send(
        &self,
        interface_rx: broadcast::Receiver<Notification>,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        use crate::interfaces::http::http_client::start_sending;

        let url = format!("http://{}:{}/notification", self.ip_or_default(), self.port);

        tokio::spawn(async move { start_sending(interface_rx, shutdown, url.as_str()).await });
        Ok(())
    }

    #[cfg(not(feature = "http-client"))]
    async fn send(
        &self,
        _interface_rx: broadcast::Receiver<Notification>,
        _shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        Err(Error::DisabledInterfaceFunction("HTTP send".to_string()))
    }
}
