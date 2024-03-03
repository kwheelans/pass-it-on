use pass_it_on::endpoints::file::FileEndpoint;
use pass_it_on::endpoints::Endpoint;
use pass_it_on::Error;
use pass_it_on::ServerConfiguration;

#[test]
fn server_valid_config_file() {
    let config = ServerConfiguration::try_from(
        r#"
    [server]
    key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

    [[server.interface]]
    type = "http"
    port = 8080

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
#[cfg(all(feature = "pipe-server", unix))]
fn server_valid_config_pipe() {
    let config = ServerConfiguration::try_from(
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
#[cfg(feature = "matrix")]
fn server_valid_config_matrix() {
    let config = ServerConfiguration::try_from(
        r#"
    [server]
    key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

    [[server.interface]]
    type = "http"
    port = 8080

    [[server.endpoint]]
    type = "file"
    path = '/test_data/file_endpoint.txt'
    notifications = ["notification1", "notification2"]

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
"#,
    );
    match config.as_ref() {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }

    assert!(config.is_ok());
}

#[test]
#[cfg(feature = "discord")]
fn server_valid_config_discord() {
    let config = ServerConfiguration::try_from(
        r#"
    [server]
    key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

    [[server.interface]]
    type = "http"
    port = 8080

    [[server.endpoint]]
    type = "discord"
    url = "https://discord.com/api/123456/asdf9874"
    username = "test_user"
    notifications = ["notification1", "notification2"]

    [[server.endpoint]]
    type = "discord"
    url = "https://discord.com/api/987456/zxcv3214"
    notifications = ["notification1", "notification2"]

    [server.endpoint.allowed_mentions]
    parse = ["everyone"]

"#,
    );
    match config.as_ref() {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }

    assert!(config.is_ok());
}

#[test]
#[cfg(feature = "email")]
fn server_valid_config_email() {
    let config = ServerConfiguration::try_from(
        r#"
    [server]
    key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

    [[server.interface]]
    type = "http"
    port = 8080

    [[server.endpoint]]
    type = "email"
    hostname = "smtp.example.com"
    port = 587
    username = "test_user"
    password = "test_password"
    from = "asdf@example.com"
    to = ["qwerty@example.com"]
    subject = "test_email"
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
fn interface_not_defined() {
    let config = ServerConfiguration::new("test key", Vec::new(), Vec::new());

    assert_eq!(config.unwrap_err().to_string(), Error::MissingInterface.to_string())
}

#[test]
fn endpoint_not_defined() {
    let notifications = ["test1".to_string(), "test2".to_string()];
    let endpoint: Box<dyn Endpoint + Send> = Box::new(FileEndpoint::new("path", notifications.as_ref()));
    let config = ServerConfiguration::new("test key", Vec::new(), vec![endpoint]);

    assert_eq!(config.unwrap_err().to_string(), Error::MissingInterface.to_string())
}

#[test]
fn file_configfile_path_is_blank() {
    let config = ServerConfiguration::try_from(
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
    let config = ServerConfiguration::try_from(
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
