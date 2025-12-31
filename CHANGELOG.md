# Unreleased

# v0.17.0
## Breaking Changes
- remove rustls-tls-native-roots feature

## Changes
- update matrix-sdk to 0.16
- update reqwest to 0.13
- update axum_server to 0.8

# v0.16.9
## Changes
- update github actions dependencies

# v0.16.8
## Changes
- update matrix-sdk to 0.14
- update docker image to debian:13-slim

# v0.16.7
## Changes 
- update matrix-sdk to 0.13
- update toml to 0.9

# v0.16.6
## Changes
- update nix to 0.30

# v0.16.5
## Changes
- update mail-send crate to 0.5
- update matrix-sdk crate to 0.11
- update rust edition to 2024

# v0.16.4
## Changes
- update matrix-sdk crate to 0.9
- update axum crate to 0.8
- update directories to 6.0

# v0.16.3
## Changes
- add derive `Debug` to `ClientConfigFileParser` and `ClientConfigFile`
- add derive `Debug` to `ServerConfigFileParser` and `ServerConfigFile`
- add `Debug` as a supertrait for `InterfaceConfig` and `EndpointConfig` traits

# v0.16.2
## Changes
- update `thiserror` dependency  to version 2.x.y
- update `matrix-sdk` dependency  to version 0.8.y

# v0.16.1
## Changes
- add `implicit_tls` configuration value for `EmailEndpoint`
- add `implicit_tls` configuration value for `EmailEndpoint`
- remove `LOG_TARGET` from pass-it-on-server bin tracing logs
- remove `LIB_LOG_TARGET` from pass-it-on library

# v0.16.0
## Breaking Changes
- remove vendored-tls feature

## Changes
- replace warp with axum_server
- migrate to tracing for logging
- use rustls across crate
- add rustls-tls-native-roots feature

## Fixes
- use json instead of body when posting notification to server

# v0.15.1
## Changes
- Update nix dependency to version 0.29
- Increase readability of matrix debug room info

# v0.15.0
## Features
- Added functionality to create and recover Matrix homeserver backups with a passphrase for `MatrixEndpoint`.
If no backup exists on the homeserver one will be created with the provided passphrase.
If a backup does exist on the homeserver then recovery will be attempted with the provided passphrase.
- Automatically enable cross signing for `MatrixEndpoint`

## Breaking Changes
- Changed session_store_password to recovery_passphrase for `MatrixEndpoint`

## Changes
- Automatically enable cross signing when building `Client` for `MatrixEndpoint`

# v0.14.4
## Changes
- Update reqwest dependency to version 0.12
- Update simple_logger dependency to version 5.0

## Fixes
- Return `ExitCode` from main to show proper exit code on error
- Return error when trying to verify matrix endpoints and no matrix endpoints are defined
- Add missing error documentation

# v0.14.3
## Changes
- Change http interface default host value from `localhost` to `0.0.0.0`

## Fixes
- Fix tls value being incorrectly set to false for http interface when https url is provided and tls value is not explicitly set
- Add `ca-certificates` package to the Dockerfile
- Fix cargo chef cook to match cargo build command

# v0.14.2
## Changes
- Revert minor dockerfile change
- Remove `bundled-sqlite` feature from `matrix-sdk` crate dependency
- Add `bundled-sqlite` feature to `pass-it-on` which turns on the `bundled-sqlite` feature for `matrix-sdk`

# v0.14.1
## Changes
- added `bundled-sqlite` feature to `matrix-sdk` crate
- fixed server configuration tests on windows

# v0.14.0
## Breaking Changes
- Server listens for notification at `POST` method at path `/pass-it-on/notifications`
- Matrix stores persistence data under provided path under `session_store_path/homerserver-domain/user`

## Features
- The `GET` method for path `/pass-it-on/version` will return version number

# v0.13.0
## Features
- Add verify matrix devices feature to the pass-it-on server

## Changes
- Matrix initial device name is now pass-it-on-server
- update nix crate to 0.28

# v0.12.0
## Changes
- Changes to update matrix_sdk to 0.7

# v0.11.0
## Breaking Changes
- The `key` field in client and server configuration can now be any length. However, both client and server must use 0.11+ of the pass-it-on library.
- Remove the `InvalidKeyLength` error as it no longer has any use.

# v0.10.1
## Changes
- Remove default server config from Dockerfile

# v0.10.0
## Changes
- bump dependencies
- Add Dockerfile

#  v0.9.0
## Features
- Expose `ClientConfigFile` and `ServerConfigFile` so that they can be included in parsing.

# v0.8.0
## Features
- Added Email endpoint functionality

## Breaking Changes
- Removed the deprecated function `from_toml` for `ClientConfiguration` and `ServerConfiguration`.

## Changes
- add restart on failure for systemd service and pass-it-on-server-sysusers.conf
- Minor update to simple_client example


# v0.7.1
## Fixes
- Fix listen for shutdown on windows

# v0.7.0
## Breaking Changes
### Library
- modify following functions to require `Option<tokio::sync::watch::Receiver<bool>>` to shut down in addition to system signals.
  - `start_client`
  - `start_client_arc`
  - `start_server`


# v0.6.1
## Dependencies
- Bump toml to 0.8
- Bump nix to 0.27

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
