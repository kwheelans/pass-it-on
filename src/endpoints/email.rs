//! Email [`Endpoint`] and [`EndpointConfig`] implementation
//!
//! # Configuration Example
//! ```toml
//! [[server.endpoint]]
//! type = "email"
//! hostname = "smtp.example.com"
//! port = 587
//! username = "test_user"
//! password = "test_password" 
//! implicit_tls = false
//! allow_invalid_certs = false
//! from = "asdf@example.com"
//! to = ["qwerty@example.com"]
//! subject = "test_email"
//! notifications = ["notification1", "notification2"]
//! ```

use crate::endpoints::{Endpoint, EndpointConfig};
use crate::notifications::{Key, ValidatedNotification};
use crate::Error;
use async_trait::async_trait;
use tracing::{debug, error, info};
use mail_send::mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;
use serde::Deserialize;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use tokio::sync::{broadcast, watch};

/// Data structure to represent the email [`EndpointConfig`].
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub(crate) struct EmailConfigFile {
    hostname: String,
    port: i64,
    username: String,
    password: String,
    #[serde(default)]
    implicit_tls: bool,
    #[serde(default)]
    allow_invalid_certs: bool,
    from: String,
    to: Vec<String>,
    subject: String,
    notifications: Vec<String>,
}

/// Data structure to represent the email [`Endpoint`].
#[derive(Debug, Clone)]
pub struct EmailEndpoint {
    hostname: String,
    port: u16,
    username: String,
    password: String,
    implicit_tls: bool,
    allow_invalid_certs: bool,
    from: String,
    to: Vec<String>,
    subject: String,
    notifications: Vec<String>,
}
#[derive(Debug, Clone)]
struct EmailInfo {
    hostname: String,
    port: u16,
    username: String,
    password: String,
    implicit_tls: bool,
    allow_invalid_certs: bool,
    from: String,
    to: Vec<String>,
    subject: String,
}

#[typetag::deserialize(name = "email")]
impl EndpointConfig for EmailConfigFile {
    fn to_endpoint(&self) -> Result<Box<dyn Endpoint + Send>, Error> {
        Ok(Box::new(EmailEndpoint::try_from(self)?))
    }
}

impl TryFrom<&EmailConfigFile> for EmailEndpoint {
    type Error = Error;

    fn try_from(value: &EmailConfigFile) -> Result<Self, Self::Error> {
        if !(value.port < u16::MAX as i64 && value.port > u16::MIN as i64) {
            return Err(Error::InvalidPortNumber(value.port));
        } else if value.to.is_empty() {
            return Err(Error::InvalidEndpointConfiguration(
                "Email configuration has no 'to' email address setup".to_string(),
            ));
        } else if value.notifications.is_empty() {
            return Err(Error::InvalidEndpointConfiguration(
                "Email configuration has no notifications setup".to_string(),
            ));
        }

        Ok(Self {
            hostname: value.hostname.clone(),
            port: value.port as u16,
            username: value.username.clone(),
            password: value.password.clone(),
            implicit_tls: value.implicit_tls,
            allow_invalid_certs: value.allow_invalid_certs,
            from: value.from.clone(),
            to: value.to.clone(),
            subject: value.subject.clone(),
            notifications: value.notifications.clone(),
        })
    }
}

#[async_trait]
impl Endpoint for EmailEndpoint {
    async fn notify(
        &self,
        endpoint_rx: broadcast::Receiver<ValidatedNotification>,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        info!("Setting up Endpoint: Email -> {}:{} from {} with subject {}", self.hostname.as_str(), self.port, self.from.as_str(), self.subject.as_str());

        let email_info = EmailInfo {
            hostname: self.hostname.clone(),
            port: self.port,
            username: self.username.clone(),
            password: self.password.clone(),
            implicit_tls: self.implicit_tls,
            allow_invalid_certs: self.allow_invalid_certs,
            from: self.from.clone(),
            to: self.to.clone(),
            subject: self.subject.clone(),
        };

        tokio::spawn(async move { send_emails(endpoint_rx, shutdown, email_info).await });

        Ok(())
    }

    fn generate_keys(&self, hash_key: &Key) -> HashMap<String, HashSet<Key>> {
        let keys: HashSet<Key> = self
            .notifications
            .iter()
            .map(|notification_name| Key::generate(notification_name.as_str(), hash_key))
            .collect();

        let mut map = HashMap::new();
        map.insert("".to_string(), keys);
        map
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

async fn send_emails(
    endpoint_rx: broadcast::Receiver<ValidatedNotification>,
    shutdown: watch::Receiver<bool>,
    info: EmailInfo,
) {
    let mut rx = endpoint_rx.resubscribe();
    let mut shutdown_rx = shutdown.clone();

    loop {
        let info = info.clone();
        tokio::select! {
            received = rx.recv() => {
                if let Ok(message) = received {
                    debug!("Email endpoint received message");

                    tokio::spawn( async move {
                        let content = message.message().text();
                        let email = MessageBuilder::new()
                        .from(info.from.as_str())
                        .subject(info.subject.as_str())
                        .to(info.to.clone())
                        .text_body(content);

                        debug!("Connecting to SMTP: {}:{} as {}", info.hostname.as_str(), info.port, info.username.as_str());
                        let mut smpt_client = SmtpClientBuilder::new(info.hostname.as_str(), info.port)
                        .implicit_tls(info.implicit_tls)
                        .credentials((info.username.as_str(), info.password.as_str()));
                        
                        if info.allow_invalid_certs {
                            smpt_client = smpt_client.allow_invalid_certs();
                        }

                        match smpt_client.connect().await {
                            Ok(mut client) => {
                                match client.send(email).await {
                                    Ok(_) => debug!("Email sent successfully"),
                                    Err(e) => error!("Unable to connect to smtp server: {}", e),
                                }
                            }
                            Err(e) => error!("Unable to send email: {}", e)
                        }
                    }).await.unwrap();

                }
            }

            _ = shutdown_rx.changed() => {
                break;
            }
        }
    }
}
