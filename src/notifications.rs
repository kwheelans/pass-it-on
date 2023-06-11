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
    pub fn new(message: &str, hash_key: &[u8; 32]) -> Notification {
        let message = Message::new(message);
        let key = message.create_key(hash_key).to_hex();
        Notification { message, key }
    }

    /// Parse single `Notification` from JSON.
    pub fn from_json(input: &str) -> Result<Notification, Error> {
        Ok(serde_json::from_str(input)?)
    }

    /// Parse multiple `Notification`s from JSON.
    pub fn from_json_multi(input: &str) -> Vec<Result<Notification, Error>> {
        let mut notifications = Vec::new();
        let stream: StreamDeserializer<_, Notification> = serde_json::Deserializer::from_str(input).into_iter();

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

    /// Compare provided notification name ['Key'] to this notification.
    pub(crate) fn validate(&self, hash_key: &Key) -> bool {
        let new_key = self.message.create_key(hash_key.hash.as_bytes());
        self.key == new_key.to_hex()
    }

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
    pub fn new(text: &str) -> Message {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let body = String::from(text);
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

    /// Create a [`Key`] for this [`Message`].
    pub fn create_key(&self, key: &[u8; 32]) -> Key {
        let mut hasher = blake3::Hasher::new_keyed(key);
        hasher.update(self.text.as_bytes());
        hasher.update(self.time.to_string().as_bytes());
        Key::from_bytes(hasher.finalize().as_bytes())
    }
}

impl ValidatedNotification {
    /// Create a new `ValidatedNotification`.
    pub fn new(name_id: String, message: Message) -> ValidatedNotification {
        Self { sub_name: name_id, message }
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
    pub fn generate(name: &str, hash_key: &[u8; 32]) -> Key {
        let mut hasher = blake3::Hasher::new_keyed(hash_key);
        hasher.update(name.as_bytes());
        Self { hash: hasher.finalize() }
    }

    /// Create `Key` from a byte array.
    pub fn from_bytes(key: &[u8; 32]) -> Key {
        let hash = Hash::from(*key);
        Self { hash }
    }

    /// Create `Key` from a hexadecimal.
    pub fn from_hex(key: &str) -> Key {
        let hash = Hash::from_hex(key).expect("Unable to create Key from hex");
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
