use crate::endpoints::matrix::MatrixRoom;
use crate::notifications::ValidatedNotification;
use crate::{Error, LIB_LOG_TARGET};
use tracing::{debug, warn};
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::{Client, Room};
use std::collections::HashSet;
use tokio::sync::{broadcast, watch};

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
