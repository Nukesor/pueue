use std::path::PathBuf;

use ciborium::Value;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while building path: {}", .0)]
    InvalidPath(String),

    /// Any errors regarding the certificate setup.
    #[error("Invalid or malformed certificate: {}", .0)]
    CertificateFailure(String),

    #[error("{}", .0)]
    Connection(String),

    #[error("Got an empty payload")]
    EmptyPayload,

    #[error("Couldn't deserialize message:\n{}", .0)]
    MessageDeserialization(String),

    #[error("Got unexpected but valid message. Are you up-to-date?:\n{:#?}", .0)]
    UnexpectedPayload(Value),

    #[error("Couldn't serialize message:\n{}", .0)]
    MessageSerialization(String),

    #[error("Requested message size of {} with only {} being allowed.", .0, .1)]
    MessageTooBig(usize, usize),

    #[error("Error while reading configuration:\n{}", .0)]
    ConfigDeserialization(String),

    #[error("Some error occurred. {}", .0)]
    Generic(String),

    #[error("I/O error while {}:\n{}", .0, .1)]
    IoError(String, std::io::Error),

    #[error("Unexpected I/O error:\n{}", .0)]
    RawIoError(#[from] std::io::Error),

    #[error("I/O error at path {:?} while {}:\n{}", .0, .1, .2)]
    IoPathError(PathBuf, &'static str, std::io::Error),

    /// Thrown if one tries to create the unix socket, but it already exists.
    /// Another daemon instance might be already running.
    #[error(
        "There seems to be an active pueue daemon.\n\
            If you're sure there isn't, please remove the \
            socket inside the pueue_directory manually."
    )]
    UnixSocketExists,
}
