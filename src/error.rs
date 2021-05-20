#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Couldn't find or open file: {}", .0)]
    FileNotFound(String),

    #[error("Error while building path: {}", .0)]
    InvalidPath(String),

    /// Any errors regarding the certificate setup.
    #[error("Invalid or malformed certificate: {}", .0)]
    CertificateFailure(String),

    #[error("Couldn't write state. {}", .0)]
    StateSave(String),

    #[error("Couldn't restore previous state. {}", .0)]
    StateRestore(String),

    #[error("Couldn't deserialize previous state:\n\n{}", .0)]
    StateDeserialization(String),

    #[error("{}", .0)]
    Connection(String),

    #[error("Got an empty payload")]
    EmptyPayload,

    #[error("Couldn't deserialize message:\n{}", .0)]
    MessageDeserialization(String),

    #[error("Couldn't serialize message:\n{}", .0)]
    MessageSerialization(String),

    #[error("Failed while building configuration.")]
    ConfigError(#[from] config::ConfigError),

    #[error("Failed while building configuration.")]
    ConfigDeserialization(String),

    #[error("Couldn't write task log file. {}", .0)]
    LogWrite(String),

    #[error("Couldn't read task log file. {}", .0)]
    LogRead(String),

    #[error("Some error occurred. {}", .0)]
    Generic(String),

    #[error("Io Error: {}", .0)]
    IoError(#[from] std::io::Error),

    /// Thrown if one tries to create the unix socket, but it already exists.
    /// Another daemon instance might be already running.
    #[error(
        "There seems to be an active pueue daemon.\n\
            If you're sure there isn't, please remove the \
            socket inside the pueue_directory manually."
    )]
    UnixSocketExists,
}
