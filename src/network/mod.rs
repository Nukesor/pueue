/// Used by the daemon to initialize the TLS certificats.
pub mod certificate;
/// This contains the main [Message](message::Message) enum and all its structs used to
/// communicate with the daemon or client.
pub mod message;
/// Platform specific code regarding sockets
mod platform;
/// This is a higher-level abstraction layer used for simple communication
/// This is probably the most interesting part for you.
pub mod protocol;
/// Functions to write and read the secret to/from a file.
pub mod secret;
/// Helper functions for reading and handling TLS files.
mod tls;
