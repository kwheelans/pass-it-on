# Pass-It-On
A library that provides a simple notification client and server that receives messages and passes them on to endpoints.


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
| HTTP      | Communication between the client and server using the HTTP protocol.                 |
| Pipe      | Communication between the client and server using a FIFO Named Pipe. (**Unix Only**) |


## Endpoints
Endpoints are the destinations for notifications received by the server.

| Endpoint     | Description                           |
|--------------|---------------------------------------|
| Regular File | Write notifications to a file.        |
| Matrix       | Send notifications to Matrix room(s). |

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
```

### Client Configuration Example
```toml
[client]
key = "sdfsf4633ghf44dfhdfhQdhdfhewaasg"

[[client.interface]]
type = "pipe"
path = '/path/to/pipe.fifo'
group_read_permission = true

[[client.interface]]
type = "http"
ip = "192.168.1.2"
port = 8080
```


## Feature Flags

| Feature      | Description                                                                                                                 |
|--------------|-----------------------------------------------------------------------------------------------------------------------------|
| client       | Enables the client but not any particular interface.                                                                        |
| endpoints    | Enables the Endpoint and EndpointConfig traits.                                                                             |
| file         | Enables the regular file endpoint.                                                                                          |
| http         | Enables the HTTP interface client and server.                                                                               |
| http-client  | Enables the HTTP interface for just client.                                                                                 |
| http-server  | Enables the HTTP interface for just server.                                                                                 |
| interfaces   | Enables the Interface and InterfaceConfig traits.                                                                           |
| matrix       | Enables the matrix endpoint.                                                                                                |
| pipe         | Enables the named pipe interface client and server. **(Unix only)**                                                         |
| pipe-client  | Enables the named pipe interface client. **(Unix only)**                                                                    |
| pipe-server  | Enables the named pipe interface server. **(Unix only)**                                                                    |
| server       | Enables the server but not any particular interface or endpoint.                                                            |
| server-bin   | Enables the building of the provided `pass-it-on-server` server binary while not require any specific interface or endpoint |
| vendored-tls | Enables vendored tls for reqwest.                                                                                           |


## Future Plans
- Add Discord webhook endpoint
- Add Email endpoint
- Enable TLS support for the server
- Make the HTTP interface path configurable instead of the hardcoded /notification
