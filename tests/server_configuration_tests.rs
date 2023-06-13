use pass_it_on::endpoints::file::FileEndpoint;
use pass_it_on::endpoints::Endpoint;
use pass_it_on::notifications::Key;
use pass_it_on::Error;
use pass_it_on::ServerConfiguration;

const VALID_KEY: &[u8; 32] = b"sdfsf4633ghf44dfhdfhQdhdfhewaasg";

#[test]
#[cfg(unix)]
fn server_valid_config_unix() {
    let config = ServerConfiguration::from_toml(
        r#"
    [server]
    key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

    [[server.interface]]
    type = "pipe"
    path = '/path/to/pipe.fifo'
    group_read_permission = true

    [[server.interface]]
    type = "http"
    port = 8080

    [[server.endpoint]]
    type = "matrix"
    home_server = "example.com"
    username = "test1"
    password = "pass"
    session_store_path = '/test_data/matrix_store'
    session_store_password = "storepass123"

    [[server.endpoint.room]]
    room = "!dfsdfsdf:example.com"
    notifications = ["notification1", "notification2"]

    [[server.endpoint]]
    type = "file"
    path = '/test_data/file_endpoint.txt'
    notifications = ["notification1", "notification2"]
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
fn server_valid_config_windows() {
    let config = ServerConfiguration::from_toml(
        r#"
    [server]
    key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

    [[server.interface]]
    type = "http"
    port = 8080

    [[server.endpoint]]
    type = "matrix"
    home_server = "example.com"
    username = "test1"
    password = "pass"
    session_store_path = '/test_data/matrix_store'
    session_store_password = "storepass123"

    [[server.endpoint.room]]
    room = "!dfsdfsdf:example.com"
    notifications = ["notification1", "notification2"]

    [[server.endpoint]]
    type = "file"
    path = '/test_data/file_endpoint.txt'
    notifications = ["notification1", "notification2"]
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
    let config = ServerConfiguration::from_toml(
        r#"
    [server]
    key = "123456789"

    [[server.interface]]
    type = "http"
    port = 8080

    [[server.endpoint]]
    type = "matrix"
    home_server = "example.com"
    username = "test1"
    password = "pass"
    session_store_path = '/test_data/matrix_store'
    session_store_password = "storepass123"

    [[server.endpoint.room]]
    room = "!dfsdfsdf:example.com"
    notifications = ["notification1", "notification2"]


    [[server.endpoints]]
    type = "file"
    path = '/test_data/file_endpoint.txt'
    notifications = ["notification1", "notification2"]
"#,
    );

    assert_eq!(config.unwrap_err().to_string(), Error::InvalidKeyLength(9).to_string())
}

#[test]
fn interface_not_defined() {
    let config = ServerConfiguration::new(Key::from_bytes(VALID_KEY), Vec::new(), Vec::new());

    assert_eq!(config.unwrap_err().to_string(), Error::MissingInterface.to_string())
}

#[test]
fn endpoint_not_defined() {
    let notifications = ["test1".to_string(), "test2".to_string()];
    let endpoint: Box<dyn Endpoint + Send> = Box::new(FileEndpoint::new("path", notifications.as_ref()));
    let config = ServerConfiguration::new(Key::from_bytes(VALID_KEY), Vec::new(), vec![endpoint]);

    assert_eq!(config.unwrap_err().to_string(), Error::MissingInterface.to_string())
}

#[test]
fn file_configfile_path_is_blank() {
    let config = ServerConfiguration::from_toml(
        r#"
    [server]
    key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

    [[server.interface]]
    type = "http"
    port = 8080

    [[server.endpoint]]
    type = "file"
    path = ''
    notifications = ["notification1", "notification2"]
"#,
    );
    assert_eq!(
        config.unwrap_err().to_string(),
        Error::InvalidEndpointConfiguration("File configuration path is blank".to_string()).to_string()
    )
}

#[test]
fn file_configfile_path_notification_is_blank() {
    let config = ServerConfiguration::from_toml(
        r#"
    [server]
    key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

    [[server.interface]]
    type = "http"
    port = 8080

    [[server.endpoint]]
    type = "file"
    path = '/test_data/file_endpoint.txt'
    notifications = []
"#,
    );
    assert_eq!(
        config.unwrap_err().to_string(),
        Error::InvalidEndpointConfiguration("File configuration has no notifications setup".to_string()).to_string()
    )
}
