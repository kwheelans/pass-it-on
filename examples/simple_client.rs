use tracing::{debug, error, info, warn};
use pass_it_on::notifications::{ClientReadyMessage, Message};
use pass_it_on::ClientConfiguration;
use pass_it_on::{start_client, Error};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, watch};
use tracing::level_filters::LevelFilter;

const NOTIFICATION_NAME: &str = "test1";
const SAMPLE_MESSAGE_COUNT: usize = 1;

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
    tracing_subscriber::fmt().with_max_level(LevelFilter::DEBUG).init();

    let config = ClientConfiguration::try_from(CLIENT_TOML_CONFIG)?;
    let messages = get_test_messages(SAMPLE_MESSAGE_COUNT, NOTIFICATION_NAME);
    let (interface_tx, interface_rx) = mpsc::channel(100);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    tokio::spawn(async move {
        info!("Sending test messages");
        for message in messages {
            match interface_tx.send(message).await {
                Ok(_) => debug!("Sent test message to client"),
                Err(error) => warn!("Unable to send test message to client: {}", error),
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Send shutdown signal
        if let Err(error) = shutdown_tx.send(true) {
            error!("Unable to send shutdown signal: {}", error)
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
