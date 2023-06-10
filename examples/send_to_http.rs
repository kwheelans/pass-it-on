use pass_it_on::notifications::Notification;
use reqwest::{Client, Error, Response};
use std::time::{SystemTime, UNIX_EPOCH};

const KEY: &[u8; 32] = b"UVXu7wtbXHWNgAr6rWyPnaZbZK9aYin8";
const NAME: &str = "test1";
const URL: &str = "http://127.0.0.1:8080/notification";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let messages = get_test_messages();

    let client = Client::new();

    for msg in messages {
        let response = send_notification(&client, URL, &msg).await;
        match response {
            Ok(response) => println!("Response: {:?}", response),
            Err(error) => println!("Error sending message: {}", error),
        }
    }

    Ok(())
}

fn get_test_messages() -> Vec<Notification> {
    let mut hasher = blake3::Hasher::new_keyed(KEY);

    hasher.update(NAME.as_bytes());
    let key = *hasher.finalize().as_bytes();
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let msg1 = format!("HTTP test message : {}", time);
    let msg2 = format!("HTTP Another message : {}", time);

    vec![Notification::new(msg1.as_str(), &key), Notification::new(msg2.as_str(), &key)]
}

async fn send_notification(client: &Client, url: &str, message: &Notification) -> Result<Response, Error> {
    client.post(url).body(message.to_json().unwrap_or_default()).send().await
}
