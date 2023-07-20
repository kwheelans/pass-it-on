# v0.6.0
## Breaking Changes
### Server Binary
- Changed default path for pass-it-on-server bin.
- Changed CLI for pass-it-on-server bin to use clap.

### Library
- removed `validate` method from `EndpointConfig` trait and changed `to_endpoint` return type to `Result<Box<dyn Endpoint + Send>, Error>`
- removed `validate` method from `InterfaceConfig` trait and changed `to_interface` return type to `Result<Box<dyn Interface + Send>, Error>`
- http configuration file now uses `host` and not `ip` to specify the ip address

## Features
- added systemd service file under resources.
- added server configuration example under resources.
- added error conversion for `Url::ParseError`
- http configuration file now uses `host` and accepts both IP addresses and URLs.
- Add TLS support for the server

# v0.5.0
## Breaking Changes
- Deprecated `from_toml` for `ClientConfiguration` and `ServerConfiguration` use `try_from` instead.

## Features
- Add Discord webhook endpoint functionality.

# v0.4.0
## Features
- Added `start_client_arc` which has the same functionality of the `start_client` function but accepts a `Arc<Mutex<Vec<ClientReadyMessage>>>`.

## Fixes
- Internal changes to client to ensure it continues to receive passed notifications.
- Fix cfg statement for pipe interface

# v0.3.3
## Fixes
- Ensure client shuts down correctly

# v0.3.2
## Fixes
- Stop looping on http and pipe client interfaces when input channel is closed

# v0.3.1
## Breaking Changes
- Notification and message structs that accepted `&[u8; 32]` have been changed to `&Key`
- `ClientConfigFileParser` is no longer exposed in the API. `ClientConfiguration::from_toml` should be used for parsing a configuration file.
- `ServerConfigFileParser` is no longer exposed in the API. `ServerConfiguration::from_toml` should be used for parsing a configuration file.
- Adds `ClientReadyMessage`
- The `start_client` function now takes a `Receiver<ClientReadyMessage>`

## Fixes
- Fix interface feature dependencies

# v0.3.0
- Published accidentally on crates.io and yanked

# v0.2.1
## Bugfix
- Fix issue with use statements for endpoints when only client is enabled

# v0.2.0
- Add vendored-tls feature
- Add missing as_bytes method to Key struct

# v0.1.0
Initial Commit
