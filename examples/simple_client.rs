use log::{debug, info, warn, LevelFilter};
use pass_it_on::notifications::Notification;
use pass_it_on::ClientConfigFileParser;
use pass_it_on::{start_client, Error};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

const NOTIFICATION_NAME: &str = "test1";
const LOG_TARGET: &str = "pass_it_on_client_example";

const CLIENT_TOML_CONFIG: &str = r#"
    [client]
    key = "UVXu7wtbXHWNgAr6rWyPnaZbZK9aYin8"

    [[client.interface]]
    type = "http"
    ip = "127.0.0.1"
    port = 8080

"#;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let module_log_level = LevelFilter::Debug;
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level(pass_it_on::LIB_LOG_TARGET, module_log_level)
        .with_module_level(LOG_TARGET, module_log_level)
        .with_colors(true)
        .init()
        .unwrap();

    let config = ClientConfigFileParser::from(CLIENT_TOML_CONFIG)?;
    let messages = get_test_messages(&config.key());
    let (interface_tx, interface_rx) = mpsc::channel(100);

    info!(target: LOG_TARGET, "Sending test messages");
    for message in messages {
        match interface_tx.send(message).await {
            Ok(_) => debug!(target: LOG_TARGET, "Sent test message to client"),
            Err(error) => warn!(target: LOG_TARGET, "Unable to send test message to client: {}", error),
        }
    }

    start_client(config, interface_rx, None).await?;

    Ok(())
}

fn get_test_messages(key: &[u8; 32]) -> Vec<Notification> {
    let mut hasher = blake3::Hasher::new_keyed(key);

    hasher.update(NOTIFICATION_NAME.as_bytes());
    let key = *hasher.finalize().as_bytes();
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let msg1 = format!("Simple Client test message test message : {}", time);
    let msg2 = format!("Simple Client Another message : {}", time);

    vec![Notification::new(msg1.as_str(), &key), Notification::new(msg2.as_str(), &key)]
}
