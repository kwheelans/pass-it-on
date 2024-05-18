use thiserror::Error;

/// Errors returned by pass-it-on library.
#[derive(Error, Debug)]
pub enum Error {
    /// At least one [`Endpoint`][`crate::endpoints::Endpoint`] needs to be defined for a server.
    #[error("At least one endpoint must be defined")]
    MissingEndpoint,

    /// At least one [`Interface`][`crate::interfaces::Interface`] needs to be defined for a server or client.
    #[error("At least one interface must be defined")]
    MissingInterface,

    /// Validation failed for an [`InterfaceConfig`][`crate::interfaces::InterfaceConfig`].
    #[error("Invalid Interface Configuration: {0}")]
    InvalidInterfaceConfiguration(String),

    /// Validation failed for an [`EndpointConfig`][`crate::endpoints::EndpointConfig`].
    #[error("Invalid Endpoint Configuration: {0}")]
    InvalidEndpointConfiguration(String),

    /// matrix room name does not appear to be a room.
    #[error("Room identifiers must start with # or !")]
    InvalidMatrixRoomIdentifier,

    /// Port needs to be in a valid u16 range.
    #[error("Invalid port number, valid u16 value expected and got {0}")]
    InvalidPortNumber(i64),

    /// Call to an interface feature that is not enabled.
    #[error("Interface feature {0} is not enabled")]
    DisabledInterfaceFeature(String),

    /// Return when an endpoint feature is called but not enabled
    #[error("Endpoint feature {0} is not enabled")]
    DisabledIEndpointFeature(String),

    // ### Converting from other error types ###
    /// Pass-thru [`std::io::Error`].
    #[error("std::io Error: {0}")]
    IOError(#[from] std::io::Error),

    /// Pass-thru `serde_json::Error`.
    #[error("Serde_json Error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[cfg(feature = "parse-cfg")]
    /// Pass-thru `toml::de::Error`.
    #[error("Serde Toml Error: {0}")]
    SerdeTomlError(#[from] toml::de::Error),

    #[cfg(feature = "matrix")]
    /// Pass-thru `matrix_sdk::Error`.
    #[error("Matrix_SDK Error: {0}")]
    MatrixSDKError(#[from] matrix_sdk::Error),

    #[cfg(feature = "matrix")]
    /// Pass-thru `matrix_sdk::ClientBuildError`.
    #[error("Matrix ClientBuild Error: {0}")]
    MatrixClientBuildError(#[from] matrix_sdk::ClientBuildError),

    #[cfg(feature = "matrix")]
    /// Pass-thru `matrix_sdk::encryption::secret_storage::SecretStorageError`.
    #[error("Matrix SecretStorage Error: {0}")]
    MatrixSecretStoreError(#[from] matrix_sdk::encryption::secret_storage::SecretStorageError),

    #[cfg(feature = "matrix")]
    /// Pass-thru `matrix_sdk::encryption::recovery::RecoveryError`.
    #[error("Matrix RecoveryError Error: {0}")]
    MatrixRecoveryError(#[from] matrix_sdk::encryption::recovery::RecoveryError),

    #[cfg(all(unix, any(feature = "pipe-client", feature = "pipe-server", feature = "pipe")))]
    /// Pass-thru `nix::errno::Errno`.
    #[error("Nix ErrorNo Error: {0}")]
    NixErrorNoError(#[from] nix::errno::Errno),

    #[cfg(any(feature = "http-client", feature = "http-server"))]
    /// Pass-thru `url::ParseError`.
    #[error("Url Parse Error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[cfg(feature = "email")]
    /// Pass-thru `mail_send::Error`.
    #[error("Mail Send Error: {0}")]
    MailSendError(#[from] mail_send::Error),
}
