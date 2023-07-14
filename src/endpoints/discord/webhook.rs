use crate::endpoints::discord::DiscordEndpoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
pub(crate) struct AllowedMentions {
    #[serde(skip_serializing_if = "Option::is_none")]
    parse: Option<Vec<MentionTypes>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    users: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replied_user: Option<bool>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub(crate) struct AllowedMentionsConfigFile {
    parse: Option<Vec<MentionTypes>>,
    roles: Option<Vec<String>>,
    users: Option<Vec<String>>,
    replied_user: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub(crate) enum MentionTypes {
    Roles,
    Users,
    Everyone,
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct WebhookPayload {
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    tts: bool,
    allowed_mentions: AllowedMentions,
}

impl WebhookPayload {
    pub fn new(content: &str, config: &DiscordEndpoint) -> Self {
        Self {
            content: content.to_string(),
            username: config.username.clone(),
            avatar_url: config.avatar_url.clone(),
            tts: config.tts,
            allowed_mentions: config.allowed_mentions.clone(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl From<AllowedMentionsConfigFile> for AllowedMentions {
    fn from(value: AllowedMentionsConfigFile) -> Self {
        Self { parse: value.parse, roles: value.roles, users: value.users, replied_user: value.replied_user }
    }
}

impl Default for AllowedMentions {
    fn default() -> Self {
        Self { parse: Some(Vec::new()), roles: None, users: None, replied_user: None }
    }
}

#[cfg(test)]
mod tests {
    use crate::endpoints::discord::webhook::WebhookPayload;

    #[test]
    fn serialize_webhook() {
        let webhook = WebhookPayload {
            content: "some message".to_string(),
            username: None,
            avatar_url: None,
            tts: false,
            allowed_mentions: Default::default(),
        };

        let result = serde_json::to_string(&webhook);

        match result {
            Ok(s) => println!("{}", s),
            Err(e) => println!("{}", e),
        }
    }
}
