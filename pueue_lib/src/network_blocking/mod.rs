//! This module contains everything that's necessary to communicate with the pueue daemon or one of
//! its clients.
//!
//! ## Sockets
//!
//! Pueue's communication can happen either via TLS encrypted TCP sockets or via UNIX sockets.
//! The mode of communication is usually specified via the configuration file and the daemon only
//! listens on a single type of socket.
//!
//! - Unix sockets are unencrypted
//! - TCP sockets are encrypted via TLS
//!
//! ## Communication
//!
//! Sending and receiving raw bytes is handled via the
//! [send_bytes](crate::network::protocol::send_bytes)
//! and [receive_bytes](crate::network::protocol::receive_bytes) functions.
//! Details on how they work can be found on the respective function docs.
//!
//! Payloads are defined via the [`Request`](crate::Request) and [`Response`](crate::Response) enums
//! that can be found in the [`super::message`] module.
//!
//! There're also the convenience functions [send_message] and [receive_message], which
//! automatically handle serialization and deserialization for you.
//! These have additional wrappers for [`Request`](crate::Request) and
//! [`Response`](crate::Response) with [`send_request`] and [`receive_response`].
//!
//! The serialization/deserialization format that's used by `pueue_lib` is [`cbor`](::ciborium).
//!
//! ## Protocol
//!
//! Before the real data exchange starts, a simple handshake + authorization is done
//! by the client and daemon.
//! An example on how to do this can be found in the Pueue's `Client::new()` function.
//!
//! The following steps are written from the client's perspective:
//!
//! - Connect to socket.
//! - Send the secret's bytes.
//! - Receive the daemon's version (utf-8 encoded), which is sent if the secret was correct.
//! - Send the actual message.
//! - Receive the daemon's response.
//!
//! In the case of most messages, the daemon is ready to receive the next the message from
//! the client, once it has send its response.
//!
//! However, some message types are special. The log `follow`, for instance, basically
//! work like a stream.
//! I.e. the daemon continuously sends new messages with the new log output until
//! the socket is closed by the client.

#[cfg(feature = "client")]
pub mod client;
/// This is probably the most interesting part for you.
pub mod protocol;
/// Low-level socket handling code.
pub mod socket;

#[cfg(feature = "client")]
pub use client::BlockingClient;
pub use protocol::{
    receive_message, receive_request, receive_response, send_message, send_request, send_response,
};
pub use socket::get_client_stream;
