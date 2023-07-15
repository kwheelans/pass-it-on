use pass_it_on::notifications::Key;
use pass_it_on::ClientConfiguration;
use pass_it_on::Error;

const VALID_KEY: &[u8; 32] = b"sdfsf4633ghf44dfhdfhQdhdfhewaasg";

#[test]
#[cfg(unix)]
fn client_valid_config_unix() {
    let config = ClientConfiguration::try_from(
        r#"
    [client]
    key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

    [[client.interface]]
    type = "pipe"
    path = '/path/to/pipe.fifo'
    group_read_permission = true

    [[client.interface]]
    type = "http"
    port = 8080

"#,
    );
    match config.as_ref() {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }

    assert!(config.is_ok());
}

#[test]
#[cfg(windows)]
fn client_valid_config_windows() {
    let config = ClientConfiguration::try_from(
        r#"
    [client]
    key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

    [[client.interface]]
    type = "http"
    port = 8080

"#,
    );
    match config.as_ref() {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }

    assert!(config.is_ok());
}

#[test]
fn server_invalid_key_length() {
    let config = ClientConfiguration::try_from(
        r#"
    [client]
    key = "asdffdsa12346785"

    [[client.interface]]
    type = "http"
    port = 8080

"#,
    );

    assert_eq!(config.unwrap_err().to_string(), Error::InvalidKeyLength(16).to_string())
}

#[test]
fn interface_not_defined() {
    let config = ClientConfiguration::new(Key::from_bytes(VALID_KEY), Vec::new());

    assert_eq!(config.unwrap_err().to_string(), Error::MissingInterface.to_string())
}
