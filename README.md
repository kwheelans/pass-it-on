# Pass-It-On
[![crates.io](https://img.shields.io/crates/v/pass-it-on)](https://crates.io/crates/pass-it-on)
[![Released API docs](https://docs.rs/pass-it-on/badge.svg)](https://docs.rs/pass-it-on/)
[![MIT licensed](https://img.shields.io/crates/l/pass-it-on)](./LICENSE)

A library that provides a simple notification client and server that receives messages and passes them on to endpoints.

## Usage
This library was designed to enable the creation of processes that handle the business of when and what notification should be sent and then pass
those notifications to the pass-it-on client which handles sending it across the configured interface to the pass-it-on server where the endpoints are configured.
The idea is to allow a single instance of the server to handle messages from many clients which may or may not be going to the same endpoint.

Which notifications are go to a particular endpoint can be controlled by adding a notification name `notifications` field in the server configuration
that matches the notification name that is used in the `ClientReadyMessage` by the client.


## Key Features 
- A configurable server that monitors interfaces and passes notifications to endpoints.
- A configurable client to use as part of a binary to send notifications to the server.
- Traits to support extension of Interfaces and their inclusion in the configuration file.
  - Interface
  - InterfaceConfig
- Traits to support extension of Endpoints and their inclusion in the configuration file.
  - Endpoint
  - EndpointConfig


## Interfaces
Interfaces can be used for both the server and the client.

| Interface | Description                                                                          |
|-----------|--------------------------------------------------------------------------------------|
| Http      | Communication between the client and server using the Http/Https protocol.           |
| Pipe      | Communication between the client and server using a FIFO Named Pipe. (**Unix Only**) |


## Endpoints
Endpoints are the destinations for notifications received by the server.

| Endpoint        | Description                                  |
|-----------------|----------------------------------------------|
| Regular File    | Write notifications to a file.               |
| Matrix          | Send notifications to Matrix room(s).        |
| Discord Webhook | Send notifications to Discord via a webhook. |

## Configuration
The Server and Client support configuration via a TOML file.
At least one interface must be setup for a Client and at least one interface and endpoint
must be setup for the Server.


### Server Configuration Example
```toml
[server]
key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

[[server.interface]]
type = "pipe"
path = '/path/to/pipe.fifo'
group_read_permission = true

[[server.interface]]
type = "http"
host = "192.168.1.2"
port = 8080

[[server.endpoint]]
type = "matrix"
home_server = "example.com"
username = "test1"
password = "pass"
session_store_path = '/path/to/session/store/matrix_store'
session_store_password = "storepass123"

[[server.endpoint.room]]
room = "!dfsdfsdf:example.com"
notifications = ["notification_id1", "notification_id2"]

[[server.endpoint.room]]
room = "#matrix-room:example.com"
notifications = ["notification_id4"]

[[server.endpoint]]
type = "file"
path = '/test_data/file_endpoint.txt'
notifications = ["notification_id1", "notification_id3"]

[[server.endpoint]]
type = "discord"
url = "https://discord.com/api/webhooks/webhook_id/webhook_token"
notifications = ["notification_id1", "notification_id3"]

[server.endpoint.allowed_mentions]
parse = ["everyone"]

```

### Client Configuration Example
```toml
[client]
key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

[[client.interface]]
type = "pipe"
path = '/path/to/pipe.fifo'
group_read_permission = true
group_write_permission = true

[[client.interface]]
type = "http"
host = "192.168.1.2"
port = 8080
```


## Feature Flags

| Feature            | Description                                                                                                            |
|--------------------|------------------------------------------------------------------------------------------------------------------------|
| client             | Enables the client but not any particular interface.                                                                   |
| discord            | Enables the discord webhook endpoint.                                                                                  |
| endpoints          | Enables the Endpoint and EndpointConfig traits.                                                                        |
| file               | Enables the regular file endpoint.                                                                                     |
| http               | Enables the Http interface client and server.                                                                          |
| http-client        | Enables the Http interface for just the client.                                                                        |
| http-server        | Enables the Http interface for just the server.                                                                        |
| interfaces         | Enables the Interface and InterfaceConfig traits.                                                                      |
| matrix             | Enables the matrix endpoint.                                                                                           |
| parse-cfg          | Enables parsing of client or server configurations from TOML when those features are also enabled.                     |
| pipe               | Enables the named pipe interface client and server. **(Unix only)**                                                    |
| pipe-client        | Enables the named pipe interface client. **(Unix only)**                                                               |
| pipe-server        | Enables the named pipe interface server. **(Unix only)**                                                               |
| server             | Enables the server but not any particular interface or endpoint.                                                       |
| server-bin-full    | Enables the building of the provided `pass-it-on-server` binary with all available interfaces and endpoints            |
| server-bin-minimal | Enables the building of the provided `pass-it-on-server` binary while not requiring any specific interface or endpoint |
| vendored-tls       | Enables vendored tls for reqwest.                                                                                      |


## Future Plans
- Add Email endpoint
- Enable encryption support for Matrix endpoint
- Make the HTTP interface path configurable instead of the hardcoded `/notification`
