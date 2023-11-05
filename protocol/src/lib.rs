use bytes::Bytes;
use serde::{Deserialize, Serialize};
pub use ssh_key;
use ssh_key::{Fingerprint, HashAlg, PrivateKey, PublicKey, SshSig};
use url::Url;
use uuid::Uuid;

/// Signature namespace for verified messages
const NAMESPACE_BUILDSRS: &str = "builder@builds.rs";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMessage {
    ChallengeRequest(Bytes),
    JobResponse(Job),
    JobList(Vec<Job>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobRequest {
    pub target: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub uuid: Uuid,
    pub target: String,
    pub name: String,
    pub version: String,
    pub download: Url,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientMessage {
    Hello(Fingerprint),
    ChallengeResponse(Bytes),
    JobRequest(JobRequest),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignedMessage<T> {
    pub message: T,
    pub signature: String,
}

#[derive(thiserror::Error, Debug)]
pub enum SignatureError {
    #[error(transparent)]
    SshKey(#[from] ssh_key::Error),
    #[error(transparent)]
    Encoding(#[from] serde_json::Error),
}

impl<T: Serialize> SignedMessage<T> {
    pub fn new(key: &PrivateKey, message: T) -> Result<Self, SignatureError> {
        let encoded = serde_json::to_string(&message)?;
        let sig = key.sign(NAMESPACE_BUILDSRS, HashAlg::Sha512, encoded.as_bytes())?;
        let signature = sig.to_pem(Default::default())?;
        Ok(Self { message, signature })
    }

    pub fn verify(&self, key: &PublicKey) -> Result<(), SignatureError> {
        let encoded = serde_json::to_string(&self.message)?;
        let signature = SshSig::from_pem(&self.signature)?;
        key.verify(NAMESPACE_BUILDSRS, &encoded.as_bytes(), &signature)?;
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
        signed.verify(&key.public_key()).unwrap();
    }
}
