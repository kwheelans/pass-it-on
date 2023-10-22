use log::{debug, error, info, warn, LevelFilter};
use pass_it_on::notifications::{ClientReadyMessage, Message};
use pass_it_on::ClientConfiguration;
use pass_it_on::{start_client, Error};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, watch};

const NOTIFICATION_NAME: &str = "test1";
const SAMPLE_MESSAGE_COUNT: usize = 1;
const LOG_TARGET: &str = "pass_it_on_client_example";

const CLIENT_TOML_CONFIG: &str = r#"
    [client]
    key = "UVXu7wtbXHWNgAr6rWyPnaZbZK9aYin8"

    [[client.interface]]
    type = "http"
    host = "127.0.0.1"
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

    let config = ClientConfiguration::try_from(CLIENT_TOML_CONFIG)?;
    let messages = get_test_messages(SAMPLE_MESSAGE_COUNT, NOTIFICATION_NAME);
    let (interface_tx, interface_rx) = mpsc::channel(100);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    tokio::spawn(async move {
        info!(target: LOG_TARGET, "Sending test messages");
        for message in messages {
            match interface_tx.send(message).await {
                Ok(_) => debug!(target: LOG_TARGET, "Sent test message to client"),
                Err(error) => warn!(target: LOG_TARGET, "Unable to send test message to client: {}", error),
            }
        }

        // Send shutdown signal
        if let Err(error) = shutdown_tx.send(true) {
            error!(target: LOG_TARGET, "Unable to send shutdown signal: {}", error)
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    });

    start_client(config, interface_rx, Some(shutdown_rx), None).await?;

    Ok(())
}

fn get_test_messages(msg_count: usize, notification_name: &str) -> Vec<ClientReadyMessage> {
    let mut messages = Vec::with_capacity(msg_count);

    for n in 1..=msg_count {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let msg = format!("Simple Client test message test message #{}: {}", n, time);
        messages.push(Message::new(msg).to_client_ready_message(notification_name))
    }
    messages
}
