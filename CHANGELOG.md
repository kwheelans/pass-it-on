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
