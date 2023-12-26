//! # Message enumeration that can be sent by either side

use crate::types::*;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use ssh_key::Fingerprint;

/// Message sent by the server.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMessage {
    /// Challenge request for authentication.
    ChallengeRequest(Bytes),
    /// New job response.
    JobResponse(Job),
    /// Currently pending jobs.
    JobList(Vec<Job>),
}

/// Messages which can be sent by the client.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientMessage {
    /// Initialize connection
    Hello(Fingerprint),
    /// Respond to challenge
    ChallengeResponse(Bytes),
    /// Request job
    JobRequest(JobRequest),
}
