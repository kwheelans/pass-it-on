use crate::endpoints::matrix::MatrixEndpoint;
use crate::{Error, LIB_LOG_TARGET};
use log::{debug, info};
use matrix_sdk::config::SyncSettings;
use matrix_sdk::matrix_auth::MatrixSession;
use matrix_sdk::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

const INITIAL_DEVICE_NAME: &str = "pass-it-on-server";
const SESSION_PATH: &str = "session";
const SESSION_FILE: &str = "matrix-session";
const SESSION_DB: &str = "db";
const SECRET_STORE_PATH: &str = "secret_store";

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct PersistentSession {
    client_session: MatrixSession,
    client_info: ClientInfo,
    sync_token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct ClientInfo {
    homeserver: Url,
    username: String,
    password: String,
    store_path: PathBuf,
    store_password: String,
}

impl ClientInfo {
    pub fn new(
        homeserver: Url,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
        store_path: &Path,
        store_password: impl AsRef<str>,
    ) -> Self {
        ClientInfo {
            homeserver,
            username: username.as_ref().to_string(),
            password: password.as_ref().to_string(),
            store_path: PathBuf::from(store_path),
            store_password: store_password.as_ref().to_string(),
        }
    }

    pub fn homeserver(&self) -> &Url {
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

    pub fn session_file_path(&self) -> PathBuf {
        self.base_store_path().join(SESSION_PATH).join(SESSION_FILE)
    }

    pub fn session_db_path(&self) -> PathBuf {
        self.base_store_path().join(SESSION_PATH).join(SESSION_DB)
    }

    pub fn secret_store_path(&self) -> PathBuf {
        self.base_store_path().join(SECRET_STORE_PATH)
    }

    fn base_store_path(&self) -> PathBuf {
        PathBuf::from(&self.store_path).join(self.homeserver().domain().unwrap_or("no-domain")).join(self.username())
    }
}

impl TryFrom<&MatrixEndpoint> for ClientInfo {
    type Error = Error;

    fn try_from(value: &MatrixEndpoint) -> Result<Self, Self::Error> {
        Ok(ClientInfo::new(
            Url::parse(value.home_server())?,
            value.username(),
            value.password(),
            value.session_store_path(),
            value.session_store_password(),
        ))
    }
}

impl PersistentSession {
    pub fn new(client_info: &ClientInfo, session: &MatrixSession, sync_token: Option<String>) -> Self {
        PersistentSession { client_info: client_info.clone(), client_session: session.clone(), sync_token }
    }

    pub fn save_session(&self) -> Result<(), Error> {
        let serde_string = serde_json::to_string(self)?;
        fs::write(self.client_info.session_file_path(), serde_string)?;
        Ok(())
    }
}

impl TryFrom<&Path> for PersistentSession {
    type Error = Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let session_string = fs::read_to_string(value)?;
        let session = serde_json::from_str(session_string.as_str())?;
        Ok(session)
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

pub(super) async fn login(client_info: ClientInfo) -> Result<Client, Error> {
    let client = {
        let build_client = Client::builder()
            .homeserver_url(client_info.homeserver())
            .sqlite_store(client_info.session_db_path(), Some(client_info.store_password()))
            .build()
            .await?;

        match client_info.session_file_path().exists() {
            true => resume_session(client_info, build_client).await?,
            false => first_login(client_info, build_client).await?,
        }
    };
    info!(target: LIB_LOG_TARGET, "logged in as: {}", client.user_id().unwrap());
    Ok(client)
}

async fn first_login(client_info: ClientInfo, client: Client) -> Result<Client, Error> {
    // Login
    client
        .matrix_auth()
        .login_username(client_info.username(), client_info.password())
        .initial_device_display_name(INITIAL_DEVICE_NAME)
        .send()
        .await?;

    let sync_token = client.sync_once(SyncSettings::default()).await?.next_batch;
    let persist = PersistentSession::new(&client_info, &client.matrix_auth().session().unwrap(), Some(sync_token));

    persist.save_session()?;
    Ok(client)
}

async fn resume_session(client_info: ClientInfo, client: Client) -> Result<Client, Error> {
    let session = PersistentSession::try_from(client_info.session_file_path().as_path())?;

    client.matrix_auth().restore_session(session.client_session).await?;

    let sync_token = client.sync_once(SyncSettings::default()).await?.next_batch;
    let persist = PersistentSession::new(&client_info, &client.matrix_auth().session().unwrap(), Some(sync_token));

    persist.save_session()?;
    Ok(client)
}
