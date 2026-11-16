//! The Soulseek wire protocol as Rust types.
//!
//! The crate provides framing, codecs for every server, peer, init and distributed message, the
//! compressed payloads and the data model. It performs no I/O and contains no async code. A
//! client that connects to the network and exchanges messages with it is `soulseek-client`.
//!
//! # Messages
//!
//! Each message family has one module: [`server`], [`peer`], [`distributed`] and [`init`]. Each
//! message is a struct whose wire layout is read and written in one place.
//!
//! The family enum ([`ServerRequest`], [`PeerMessage`], ...) ties every variant to its code. Its
//! `encode(&self) -> Vec<u8>` method produces a complete frame, length prefix included. Its
//! `decode(&[u8])` method takes exactly one complete frame, length prefix included: the bytes
//! [`frame::split`] reports as consumed.
//!
//! # Decoding
//!
//! `decode` enforces [`Limits::DEFAULT`]. [`PeerMessage::decode_with`],
//! [`peek_search_response`] and [`peek_folder_contents_token`] take the limits as an argument.
//! The configurable limits bound only peer messages, so a client with limits of its own decodes
//! peer frames with [`PeerMessage::decode_with`]; [`PeerMessage::decode`] ignores them.
//!
//! # Logging
//!
//! Decoders log rejected messages at the `debug` level, and decoded messages at the `debug`
//! level except distributed pings, distributed searches and embedded messages, which are too
//! frequent to log. A partial frame or bytes past one frame are logged at the `warn` level. A
//! frame length out of range is returned without being logged, and is left to the caller to log.
//!
//! # Features
//!
//! `serde`, off by default, implements `Serialize` and `Deserialize` for the message structs and
//! the data model: files, directories, users, rooms, [`RemoteFilename`], [`SearchQuery`] and
//! [`Limits`]. The message family enums, the frame types, the errors, [`Secret`] and the
//! messages that hold one have no implementations.

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctest;

mod codec;
mod codes;
pub mod distributed;
mod error;
pub mod frame;
pub mod init;
mod limits;
mod log;
mod model;
mod names;
pub mod peer;
pub mod server;
mod wire;

#[cfg(test)]
mod proptests;
#[cfg(test)]
mod testing;

pub use codes::{DistributedCode, InitCode, PeerCode, ServerCode};
pub use distributed::{DistributedMessage, SEARCH_IDENTIFIER};
pub use error::{DecodeError, FieldError};
pub use init::InitMessage;
pub use limits::*;
pub use model::*;
pub use names::{
  LoginUsernameError, RoomNameError, SERVER_USERNAME, validate_login_username, validate_room_name,
};
pub use peer::{PeerMessage, peek_folder_contents_token, peek_search_response, queue_count};
pub use server::{PROTOCOL_MAJOR_VERSION, PROTOCOL_MINOR_VERSION, ServerRequest, ServerResponse};
pub use wire::{RemoteFilename, Secret};
