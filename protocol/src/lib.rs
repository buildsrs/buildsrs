//! # Buildsrs Protocol
//!
//! This crate supplies the necessary primitives that defined the protocol between the builder
//! and the backend. The [`ServerMessage`] defines any messages that might be sent by the
//! server, and the [`ClientMessage`] any message sent by the client. Every message from the
//! client is wrapped in a [`SignedMessage`] to add a cryptographic signature.

pub use ssh_key;

pub mod messages;
pub mod signature;
pub mod types;

pub use crate::{
    messages::*,
    signature::*,
    types::{Job, JobRequest},
};
