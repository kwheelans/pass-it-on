//! Representation of notification messages.

use crate::Error;
use blake3::Hash;
use serde::{Deserialize, Serialize};
use serde_json::StreamDeserializer;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// The actual message data that is being transmitted in a [`Notification`].
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct Message {
    text: String,
    time: u128,
}

/// A [`Message`] that has been assigned a notification name
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ClientReadyMessage {
    message: Message,
    notification_name: String,
}

/// [`Notification`] that has been validated against identified as a particular notification name.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ValidatedNotification {
    message: Message,
    sub_name: String,
}

/// Notification ready to be send with [`Message`] data and a hash value for validation.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct Notification {
    message: Message,
    key: String,
}

/// Convenience wrapper around a [BLAKE3] [`Hash`] used for validation.
///
/// [BLAKE3]: https://crates.io/crates/blake3
/// [`Hash`]: https://docs.rs/blake3/latest/blake3/struct.Hash.html
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Key {
    hash: Hash,
}

impl Notification {
    /// Create a new `Notification` from a text value and key for notification name.
    pub fn new(message: Message, notification_key: &Key) -> Notification {
        let key = message.create_key(notification_key).to_hex();
        Notification { message, key }
    }

    /// Parse single `Notification` from JSON.
    pub fn from_json<S: AsRef<str>>(input: S) -> Result<Notification, Error> {
        Ok(serde_json::from_str(input.as_ref())?)
    }

    /// Parse multiple `Notification`s from JSON.
    pub fn from_json_multi<S: AsRef<str>>(input: S) -> Vec<Result<Notification, Error>> {
        let mut notifications = Vec::new();
        let stream: StreamDeserializer<_, Notification> =
            serde_json::Deserializer::from_str(input.as_ref()).into_iter();

        for item in stream {
            match item {
                Err(e) => notifications.push(Err(Error::SerdeJsonError(e))),
                Ok(n) => notifications.push(Ok(n)),
            };
        }
        notifications
    }

    /// Serialize `Notification` to JSON.
    pub fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(self)?)
    }

    #[allow(dead_code)]
    /// Compare provided notification name ['Key'] to this notification.
    pub(crate) fn validate(&self, hash_key: &Key) -> bool {
        let new_key = self.message.create_key(hash_key);
        self.key == new_key.to_hex()
    }

    #[allow(dead_code)]
    /// Compare provided set of  notification name ['Key']s to this notification.
    pub(crate) fn validate_set(&self, hash_keys: &HashSet<Key>) -> bool {
        for hash_key in hash_keys {
            match self.validate(hash_key) {
                true => return true,
                false => (),
            }
        }
        false
    }

    /// Return inner [`Message`].
    pub fn message(&self) -> Message {
        self.message.clone()
    }

    /// Return notification key as a string slice.
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl Message {
    /// Create a new `Message` from provide text.
    pub fn new<S: AsRef<str>>(text: S) -> Message {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let body = String::from(text.as_ref());
        Self { text: body, time }
    }

    /// Return inner text value.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return time when this `Message` was created.
    pub fn time(&self) -> u128 {
        self.time
    }

    /// Create a [`Key`] for this [`Message`] based on the [`Key`] for the notification name.
    fn create_key(&self, notification_key: &Key) -> Key {
        let hash_string = format!("{}{}", self.text, self.time);
        Key::generate(hash_string.as_str(), notification_key)
    }

    /// Assign a notification name to a [`Message`] and transform it into a [`ClientReadyMessage`]
    pub fn to_client_ready_message<S: AsRef<str>>(self, notification_name: S) -> ClientReadyMessage {
        ClientReadyMessage::new(notification_name, self)
    }
}

impl ClientReadyMessage {
    /// Create a new `ClientReadyMessage`
    pub(crate) fn new<S: AsRef<str>>(notification_name: S, message: Message) -> Self {
        Self { notification_name: notification_name.as_ref().into(), message }
    }

    /// Create a [`Notification`] for the contained [`Message`] based on the notification name and client [`Key`]
    pub fn to_notification(self, client_key: &Key) -> Notification {
        let key = Key::generate(self.notification_name(), client_key);
        let message = self.message;
        Notification::new(message, &key)
    }

    /// Return inner [`Message`]
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Return assigned notification name
    pub fn notification_name(&self) -> &str {
        &self.notification_name
    }
}

impl ValidatedNotification {
    /// Create a new `ValidatedNotification`.
    pub fn new<S: AsRef<str>>(name_id: S, message: Message) -> ValidatedNotification {
        Self { sub_name: name_id.as_ref().into(), message }
    }

    /// Return inner [`Message`] value.
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Return the sub-name for this `ValidatedNotification`.
    pub fn sub_name(&self) -> &str {
        &self.sub_name
    }
}

impl Key {
    /// Generate a new keyed hash based on the provide notification name.
    pub fn generate<S: AsRef<str>>(name: S, hash_key: &Key) -> Key {
        let mut hasher = blake3::Hasher::new_keyed(hash_key.as_bytes());
        hasher.update(name.as_ref().as_bytes());
        Self { hash: hasher.finalize() }
    }

    /// Create `Key` from a byte array.
    pub fn from_bytes(key: &[u8; 32]) -> Key {
        let hash = Hash::from(*key);
        Self { hash }
    }

    /// Create `Key` from a hexadecimal.
    pub fn from_hex<S: AsRef<str>>(key: S) -> Key {
        let hash = Hash::from_hex(key.as_ref()).expect("Unable to create Key from hex");
        Self { hash }
    }

    /// Return `Key` as a hexadecimal.
    pub fn to_hex(&self) -> String {
        self.hash.to_hex().to_string()
    }

    /// Return `Key` as a byte array.
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.hash.as_bytes()
    }
}
