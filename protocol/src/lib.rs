//! # Buildsrs Protocol
//!
//! This crate supplies the necessary primitives that defined the protocol between the builder
//! and the backend. The [`ServerMessage`] defines any messages that might be sent by the
//! server, and the [`ClientMessage`] any message sent by the client. Every message from the
//! client is wrapped in a [`SignedMessage`] to add a cryptographic signature.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
pub use ssh_key;
use ssh_key::{Fingerprint, HashAlg, PrivateKey, PublicKey, SshSig};
use url::Url;
use uuid::Uuid;

/// Signature namespace for verified messages
const NAMESPACE_BUILDSRS: &str = "builder@builds.rs";

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

/// Request a job from server.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobRequest {
    /// Target for this job.
    pub target: String,
}

/// Job information.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    /// UUID of job.
    pub uuid: Uuid,
    /// Target triple
    pub target: String,
    /// Name of crate
    pub name: String,
    /// Version of crate
    pub version: String,
    /// URL to download crate from.
    pub download: Url,
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

/// Message which has been signed with a cryptographic signature.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignedMessage<T> {
    /// Message
    pub message: T,
    /// Signature, created from SSH key.
    pub signature: String,
}

/// Error creating or verifying [`SignedMessage`].
#[derive(thiserror::Error, Debug)]
pub enum SignatureError {
    /// Error in SSH key
    #[error(transparent)]
    SshKey(#[from] ssh_key::Error),
    /// Error encoding message
    #[error(transparent)]
    Encoding(#[from] serde_json::Error),
}

impl<T: Serialize> SignedMessage<T> {
    /// Create new signed message.
    pub fn new(key: &PrivateKey, message: T) -> Result<Self, SignatureError> {
        let encoded = serde_json::to_string(&message)?;
        let sig = key.sign(NAMESPACE_BUILDSRS, HashAlg::Sha512, encoded.as_bytes())?;
        let signature = sig.to_pem(Default::default())?;
        Ok(Self { message, signature })
    }

    /// Verify that this message was signed by the supplied public key.
    pub fn verify(&self, key: &PublicKey) -> Result<(), SignatureError> {
        let encoded = serde_json::to_string(&self.message)?;
        let signature = SshSig::from_pem(&self.signature)?;
        key.verify(NAMESPACE_BUILDSRS, encoded.as_bytes(), &signature)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;
    use ssh_key::Algorithm;

    #[test]
    fn test_signed_message() {
        let message = "Hello";
        let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let signed = SignedMessage::new(&key, message).unwrap();
        signed.verify(key.public_key()).unwrap();
    }
}
