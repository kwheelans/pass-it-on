use log::{debug, info, warn};
use matrix_sdk::config::SyncSettings;
use matrix_sdk::matrix_auth::MatrixSession;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::{Client, Room};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::{broadcast, watch};

use crate::endpoints::matrix::{MatrixEndpoint, MatrixRoom};
use crate::notifications::ValidatedNotification;
use crate::{Error, LIB_LOG_TARGET};

const INITIAL_DEVICE_NAME: &str = "pass-it-on-server";
const SESSION_FILE_NAME: &str = "session_file";
const SESSION_DB_NAME: &str = "session_db";

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct PersistentSession {
    client_session: MatrixSession,
    client_info: ClientInfo,
    sync_token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct ClientInfo {
    homeserver: String,
    username: String,
    password: String,
    store_path: PathBuf,
    store_password: String,
}

impl ClientInfo {
    pub fn new(
        homeserver: impl AsRef<str>,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
        store_path: &Path,
        store_password: impl AsRef<str>,
    ) -> Self {
        ClientInfo {
            homeserver: homeserver.as_ref().to_string(),
            username: username.as_ref().to_string(),
            password: password.as_ref().to_string(),
            store_path: PathBuf::from(store_path),
            store_password: store_password.as_ref().to_string(),
        }
    }

    pub fn from(config: &MatrixEndpoint) -> Self {
        ClientInfo::new(
            config.home_server(),
            config.username(),
            config.password(),
            config.session_store_path(),
            config.session_store_password(),
        )
    }
    pub fn homeserver(&self) -> &str {
        &self.homeserver
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn store_password(&self) -> &str {
        &self.store_password
    }

    pub fn session_path(&self) -> PathBuf {
        PathBuf::from(&self.store_path).join(SESSION_FILE_NAME)
    }

    pub fn db_path(&self) -> PathBuf {
        PathBuf::from(&self.store_path).join(SESSION_DB_NAME)
    }
}

impl PersistentSession {
    pub fn new(client_info: &ClientInfo, session: &MatrixSession, sync_token: Option<String>) -> Self {
        PersistentSession { client_info: client_info.clone(), client_session: session.clone(), sync_token }
    }
}

pub(super) async fn send_messages(
    endpoint_rx: broadcast::Receiver<ValidatedNotification>,
    shutdown: watch::Receiver<bool>,
    room_list: Vec<Room>,
    client: &Client,
) -> String {
    let mut rx = endpoint_rx.resubscribe();
    let mut shutdown_rx = shutdown.clone();
    let mut sync_token = client.sync_once(SyncSettings::default()).await.unwrap().next_batch;
    let client_homeserver = get_default_server(client);

    loop {
        tokio::select! {
            received = rx.recv() => {
                if let Ok(message) = received {
                    debug!(target: LIB_LOG_TARGET, "Matrix message received: {} Name: {}", message.message().text(), message.sub_name());
                    let msg_text = RoomMessageEventContent::text_plain(message.message().text());

                    if let Ok(msg_room) = validate_room(message.sub_name(), client_homeserver.as_str()) {
                        for room in &room_list {
                            if get_all_room_aliases(room).contains(msg_room.as_str()) {
                                debug!(target: LIB_LOG_TARGET, "Sending Matrix Message to {}", msg_room);
                                match room.send(msg_text.clone()).await {
                                    Ok(r) => debug!(target: LIB_LOG_TARGET, "OK: {:?}", r),
                                    Err(e) => debug!(target: LIB_LOG_TARGET, "Error: {}", e),
                                }
                            }
                        }
                    }
                    sync_token = client.sync_once(SyncSettings::default().token(&sync_token)).await.unwrap().next_batch;
                }
            }

            _ = shutdown_rx.changed() => {
                sync_token = client.sync_once(SyncSettings::default().token(&sync_token)).await.unwrap().next_batch;
                break;
            }
        }
    }
    sync_token
}

pub(super) async fn login(client_info: ClientInfo) -> Result<Client, Error> {
    let client = match client_info.session_path().exists() {
        true => resume_session(client_info).await?,
        false => first_login(client_info).await?,
    };
    Ok(client)
}

pub(super) async fn first_login(client_info: ClientInfo) -> Result<Client, Error> {
    let client = Client::builder()
        .homeserver_url(client_info.homeserver())
        .sqlite_store(client_info.db_path(), Some(client_info.store_password()))
        .build()
        .await?;

    // Login
    client
        .matrix_auth()
        .login_username(client_info.username(), client_info.password())
        .initial_device_display_name(INITIAL_DEVICE_NAME)
        .send()
        .await?;
    info!(target: LIB_LOG_TARGET, "logged in as: {}", client.user_id().unwrap());

    let sync_token = client.sync_once(SyncSettings::default()).await.unwrap().next_batch;
    let persist = PersistentSession::new(&client_info, &client.matrix_auth().session().unwrap(), Some(sync_token));

    save_session(&persist)?;
    Ok(client)
}

pub(super) async fn resume_session(client_info: ClientInfo) -> Result<Client, Error> {
    let client = Client::builder()
        .homeserver_url(client_info.homeserver())
        .sqlite_store(client_info.db_path(), Some(client_info.store_password()))
        .build()
        .await?;
    let session = parse_session(client_info.session_path().as_path())?;

    client.matrix_auth().restore_session(session.client_session).await?;
    info!(target: LIB_LOG_TARGET, "logged in as: {}", client.user_id().unwrap());

    let sync_token = client.sync_once(SyncSettings::default()).await.unwrap().next_batch;
    let persist = PersistentSession::new(&client_info, &client.matrix_auth().session().unwrap(), Some(sync_token));

    save_session(&persist)?;
    Ok(client)
}

fn parse_session(session_path: &Path) -> Result<PersistentSession, Error> {
    let session_string = fs::read_to_string(session_path)?;
    let session = serde_json::from_str(session_string.as_str())?;
    Ok(session)
}

pub(super) fn save_session(session: &PersistentSession) -> Result<(), Error> {
    let serde_string = serde_json::to_string(session)?;
    fs::write(session.client_info.session_path(), serde_string)?;
    Ok(())
}

fn validate_room(room: &str, default_server: &str) -> Result<String, Error> {
    let room = room.trim();
    if room.starts_with('!') || room.starts_with('#') {
        if !room.contains(':') {
            let mut room = String::from(room);
            room.push(':');
            room.push_str(default_server);
            Ok(room)
        } else {
            Ok(room.to_string())
        }
    } else {
        warn!(target: LIB_LOG_TARGET, "{} is not a valid room", room);
        Err(Error::InvalidMatrixRoomIdentifier)
    }
}

pub(super) async fn process_rooms(client: &Client, room_map: &[MatrixRoom]) -> Vec<Room> {
    let default_server = get_default_server(client);
    let joined_rooms = client.joined_rooms();
    let mut valid_rooms = Vec::new();

    for matrix_room in room_map {
        match validate_room(matrix_room.room(), default_server.as_ref()) {
            Err(e) => warn!(target: LIB_LOG_TARGET, "{}: {}", e, matrix_room.room()),
            Ok(valid_room) => {
                for known_room in &joined_rooms {
                    let room_alias = get_all_room_aliases(known_room);
                    if room_alias.contains(valid_room.as_str()) {
                        valid_rooms.push(known_room.clone());
                    }
                }
            }
        }
    }
    valid_rooms
}

fn get_all_room_aliases(room: &Room) -> HashSet<String> {
    let mut room_alias: HashSet<_> = room.alt_aliases().into_iter().map(|alias| alias.to_string()).collect();
    if let Some(cannon_alias) = room.canonical_alias() {
        room_alias.insert(cannon_alias.to_string());
    }
    room_alias.insert(room.room_id().to_string());
    room_alias
}

fn get_default_server(client: &Client) -> String {
    match client.user_id() {
        None => String::default(),
        Some(id) => id.server_name().to_string(),
    }
}

pub(super) async fn print_client_debug(client: &Client) {
    let uid = client.user_id().unwrap();
    let device = client.encryption().get_own_device().await.unwrap().unwrap();
    let csstatus = client.encryption().cross_signing_status().await.unwrap();
    let homeserver = client.homeserver();
    debug!(target: LIB_LOG_TARGET, "==================================================");
    debug!(target: LIB_LOG_TARGET, "User ID: {}", uid);
    debug!(target: LIB_LOG_TARGET, "User Servername: {}", uid.server_name());
    debug!(target: LIB_LOG_TARGET, "Device ID: {}", device.device_id());
    debug!(target: LIB_LOG_TARGET, "Device Display Name: {}", device.display_name().unwrap_or_default());
    debug!(target: LIB_LOG_TARGET, "Device is verified: {}", device.is_verified());
    debug!(target: LIB_LOG_TARGET, "Device is cross-signed: {}", device.is_cross_signed_by_owner());

    debug!(target: LIB_LOG_TARGET, "Homeserver: {}", homeserver);
    debug!(target: LIB_LOG_TARGET, "Cross Signing status: {:?}", csstatus);

    let rooms = client.joined_rooms();
    debug!(target: LIB_LOG_TARGET, "==================================================");
    if rooms.is_empty() {
        debug!(target: LIB_LOG_TARGET, "No rooms have been joined");
    }
    for r in rooms {
        debug!(target: LIB_LOG_TARGET, "Room Name: {:?}", r.name().unwrap_or_default());
        debug!(target: LIB_LOG_TARGET, "Canonical Alias: {:?}", r.canonical_alias());
        debug!(target: LIB_LOG_TARGET, "Room ID: {:?}", r.room_id());
        debug!(target: LIB_LOG_TARGET, "Alt Alias: {:?}", r.alt_aliases());
        debug!(target: LIB_LOG_TARGET, "is encrypted: {:?}", r.is_encrypted().await);
        debug!(target: LIB_LOG_TARGET, "is public: {:?}", r.is_public());
        debug!(target: LIB_LOG_TARGET, "is direct: {:?}", r.is_direct().await);
        debug!(target: LIB_LOG_TARGET, "is tombstoned: {:?}", r.is_tombstoned());
    }
    debug!(target: LIB_LOG_TARGET, "==================================================");
}
