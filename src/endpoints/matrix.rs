//! Matrix [`Endpoint`] and [`EndpointConfig`] implementation
//!
//! ```toml
//! [[server.endpoint]]
//! type = "matrix"
//! home_server = "example.com"
//! username = "test1"
//! password = "password"
//! session_store_path = '/path/to/session/store/matrix_store'
//! session_store_password = "storepassword"
//!
//!
//! [[server.endpoint.room]]
//! room = "#matrix-room:example.com"
//! notifications = ["notification_id1"]
//!
//! [[server.endpoint.room]]
//! room = "#another-room:example.com"
//! notifications = ["notification_id2"]
//! ```

pub(crate) mod notify;

use crate::endpoints::matrix::notify::{
    login, print_client_debug, process_rooms, save_session, send_messages, ClientInfo, PersistentSession,
};
use crate::endpoints::{Endpoint, EndpointConfig};
use crate::notifications::{Key, ValidatedNotification};
use crate::{Error, LIB_LOG_TARGET};
use async_trait::async_trait;
use log::{error, info};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tokio::sync::broadcast::Receiver;
use tokio::sync::watch;

/// Data structure to represent the Matrix [`EndpointConfig`].
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub(crate) struct MatrixConfigFile {
    home_server: String,
    username: String,
    password: String,
    session_store_path: String,
    session_store_password: String,
    room: Vec<MatrixRoomConfigFile>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub(crate) struct MatrixRoomConfigFile {
    room: String,
    notifications: Vec<String>,
}

/// Data structure to represent the Matrix [`Endpoint`].
#[derive(Debug, Clone)]
pub struct MatrixEndpoint {
    home_server: String,
    username: String,
    password: String,
    session_store_path: PathBuf,
    session_store_password: String,
    rooms: Vec<MatrixRoom>,
}

/// Data structure to represent a Matrix room.
#[derive(Debug, Clone)]
pub struct MatrixRoom {
    room: String,
    notifications: HashSet<String>,
}

impl MatrixConfigFile {
    fn rooms(&self) -> HashMap<String, HashSet<String>> {
        let mut room_map: HashMap<String, HashSet<String>> = HashMap::new();
        for room in &self.room {
            match room_map.get(room.room.as_str()) {
                None => room_map.insert(room.room.to_string(), room.notifications()),
                Some(notifications) => {
                    let new_notifications = room.notifications();
                    let union: HashSet<_> = new_notifications.union(notifications).collect();
                    let union: HashSet<_> = union.into_iter().map(|s| s.to_string()).collect();
                    room_map.insert(room.room.to_string(), union)
                }
            };
        }
        room_map
    }
}

impl MatrixRoomConfigFile {
    fn notifications(&self) -> HashSet<String> {
        let notifications: HashSet<_> = self.notifications.clone().into_iter().collect();
        notifications
    }
}

#[typetag::deserialize(name = "matrix")]
impl EndpointConfig for MatrixConfigFile {
    fn to_endpoint(&self) -> Box<dyn Endpoint + Send> {
        let home_server = self.home_server.as_str();
        let username = self.username.as_str();
        let password = self.password.as_str();
        let session_store_path = self.session_store_path.as_str();
        let session_store_password = self.session_store_password.as_str();
        let rooms = {
            let mut rooms: Vec<_> = Vec::new();
            for (room, notifications) in self.rooms() {
                rooms.push(MatrixRoom::new(room, notifications));
            }
            rooms
        };

        Box::new(MatrixEndpoint::new(
            home_server,
            username,
            password,
            session_store_path,
            session_store_password,
            rooms,
        ))
    }

    fn validate(&self) -> Result<(), Error> {
        if self.home_server.is_empty() {
            return Err(Error::InvalidEndpointConfiguration("Matrix configuration home_server is blank".to_string()));
        }

        if self.username.is_empty() {
            return Err(Error::InvalidEndpointConfiguration("Matrix configuration username is blank".to_string()));
        }

        if self.room.is_empty() {
            return Err(Error::InvalidEndpointConfiguration("Matrix configuration has no rooms setup".to_string()));
        }

        Ok(())
    }
}

impl MatrixEndpoint {
    /// Create a new `MatrixEndpoint`.
    pub fn new(
        home_server: &str,
        username: &str,
        password: &str,
        session_store_path: &str,
        session_store_password: &str,
        rooms: Vec<MatrixRoom>,
    ) -> Self {
        let home_server = home_server.to_string();
        let username = username.to_string();
        let password = password.to_string();
        let session_store_path = PathBuf::from(session_store_path);
        let session_store_password = session_store_password.to_string();
        Self { home_server, username, password, session_store_path, session_store_password, rooms }
    }

    /// Return the matrix home server.
    pub fn home_server(&self) -> &str {
        &self.home_server
    }

    /// Return the matrix username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Return the password for the matrix user.
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Return the path to the persistent session store.
    pub fn session_store_path(&self) -> &PathBuf {
        &self.session_store_path
    }

    /// Return the password for persistent session store database.
    pub fn session_store_password(&self) -> &str {
        &self.session_store_password
    }

    /// Return the matrix rooms setup for this matrix endpoint.
    pub fn rooms(&self) -> &[MatrixRoom] {
        &self.rooms
    }
}

impl MatrixRoom {
    /// Create a new `MatrixRoom`.
    pub fn new(room: String, notifications: HashSet<String>) -> Self {
        Self { room, notifications }
    }

    /// Return the matrix room name.
    pub fn room(&self) -> &str {
        &self.room
    }

    /// Return notification names associated with this room.
    pub fn notifications(&self) -> &HashSet<String> {
        &self.notifications
    }
}

#[async_trait]
impl Endpoint for MatrixEndpoint {
    async fn notify(
        &self,
        endpoint_rx: Receiver<ValidatedNotification>,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(), Error> {
        // Login client
        let client_info = ClientInfo::from(self);
        info!(
            target: LIB_LOG_TARGET,
            "Setting up Endpoint: Matrix -> User {} on {}",
            client_info.username(),
            client_info.homeserver()
        );
        let client = login(client_info.clone()).await?;

        print_client_debug(&client).await;
        let room_list = process_rooms(&client, self.rooms()).await;

        // Monitor for messages to send
        tokio::spawn(async move {
            let sync_token = send_messages(endpoint_rx, shutdown.clone(), room_list, &client).await;
            let persist = PersistentSession::new(&client_info, &client.session().unwrap(), Some(sync_token));
            if let Err(error) = save_session(&persist) {
                error!(target: LIB_LOG_TARGET, "{}", error)
            }
        });

        Ok(())
    }

    fn generate_keys(&self, hash_key: &Key) -> HashMap<String, HashSet<Key>> {
        let mut keys: HashMap<String, HashSet<Key>> = HashMap::new();

        for room in self.rooms() {
            let mut room_keys = HashSet::new();
            for notification_name in room.notifications() {
                room_keys.insert(Key::generate(notification_name, hash_key));
            }
            keys.insert(room.room().to_string(), room_keys);
        }
        keys
    }
}
