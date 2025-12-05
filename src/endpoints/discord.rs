//! Discord webhook [`Endpoint`] and [`EndpointConfig`] implementation
//!
//!
//! # Configuration Example
//! ```toml
//! [[server.endpoint]]
//! type = "discord"
//! url = "https://discord.com/api/webhooks/webhook_id/webhook_token"
//! username = "Bot Name"
//! avatar_url = "https://example.com/avatar/url"
//! tts = true
//! notifications = ["notification_id1", "notification_id3"]
//!
//! [server.endpoint.allowed_mentions]
//! parse = ["everyone"]
//! roles = ["role1"]
//! users = ["user1"]
//! replied_user = false
//! ```

pub(crate) mod webhook;

use crate::endpoints::discord::webhook::{AllowedMentions, AllowedMentionsConfigFile, WebhookPayload};
use crate::endpoints::{Endpoint, EndpointConfig};
use crate::notifications::{Key, ValidatedNotification};
use crate::Error;
use async_trait::async_trait;
use tracing::{debug, info, warn};
use reqwest::Client;
use serde::Deserialize;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use tokio::sync::broadcast::Receiver;
use tokio::sync::watch;

/// Data structure to represent the Discord webhook [`EndpointConfig`].
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub(crate) struct DiscordConfigFile {
    url: String,
    username: Option<String>,
    avatar_url: Option<String>,
    #[serde(default)]
    tts: bool,
    notifications: Vec<String>,
    allowed_mentions: Option<AllowedMentionsConfigFile>,
}

/// Data structure to represent the Discord webhook [`Endpoint`].
#[derive(Debug, Clone)]
pub struct DiscordEndpoint {
    url: String,
    username: Option<String>,
    avatar_url: Option<String>,
    tts: bool,
    notifications: Vec<String>,
    allowed_mentions: AllowedMentions,
}

#[typetag::deserialize(name = "discord")]
impl EndpointConfig for DiscordConfigFile {
    fn to_endpoint(&self) -> Result<Box<dyn Endpoint + Send>, Error> {
        Ok(Box::new(DiscordEndpoint::try_from(self)?))
    }
}

#[async_trait]
impl Endpoint for DiscordEndpoint {
    async fn notify(
        &self,
        endpoint_rx: Receiver<ValidatedNotification>,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        info!("Setting up Endpoint: Discord -> {}", self.url);
        let discord = self.clone();
        tokio::spawn(async move { send_messages(endpoint_rx, shutdown, discord).await });
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

impl TryFrom<&DiscordConfigFile> for DiscordEndpoint {
    type Error = Error;

    fn try_from(value: &DiscordConfigFile) -> Result<Self, Self::Error> {
        if value.url.is_empty() {
            return Err(Error::invalid_endpoint_configuration("Discord configuration url is blank".to_string()));
        }
        if value.notifications.is_empty() {
            return Err(Error::invalid_endpoint_configuration(
                "Discord configuration has no notifications setup".to_string(),
            ));
        }
        let allowed_mentions = value.allowed_mentions.clone().map_or(AllowedMentions::default(), AllowedMentions::from);
        Ok(Self {
            url: value.url.clone(),
            username: value.username.clone(),
            avatar_url: value.avatar_url.clone(),
            tts: value.tts,
            allowed_mentions,
            notifications: value.notifications.clone(),
        })
    }
}

async fn send_messages(
    endpoint_rx: Receiver<ValidatedNotification>,
    shutdown: watch::Receiver<bool>,
    discord: DiscordEndpoint,
) {
    let mut rx = endpoint_rx.resubscribe();
    let mut shutdown_rx = shutdown.clone();
    let client = Client::new();

    loop {
        tokio::select! {
            received = rx.recv() => {
                if let Ok(message) = received {
                    let content = message.message().text();
                    let payload = WebhookPayload::new(content, &discord);
                    debug!("Discord Webhook Payload: {}", payload.to_json());
                    let response = client.post(&discord.url)
                    .header("content-type", "application/json")
                    .body(payload.to_json())
                    .send().await;
                    match response {
                            Ok(ok) => debug!("Discord Webhook Response - status: {} url: {}", ok.status(), ok.url()),
                            Err(error) => warn!("Discord Webhook Response Error: {}", error ),
                        }
                }
            }

            _ = shutdown_rx.changed() => {
                break;
            }
        }
    }
}
