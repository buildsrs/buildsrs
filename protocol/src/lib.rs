use bytes::Bytes;
use serde::{Deserialize, Serialize};
use ssh_key::{Fingerprint, HashAlg, PrivateKey, PublicKey, SshSig};
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMessage {
    ChallengeRequest(Bytes),
    JobResponse(Job),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobRequest {
    pub arch: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub arch: String,
    pub crate_name: String,
    pub crate_version: String,
    pub crate_url: Url,
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
        let sig = key.sign("builder@builds.rs", HashAlg::Sha512, encoded.as_bytes())?;
        let signature = sig.to_pem(Default::default())?;
        Ok(Self { message, signature })
    }

    pub fn verify(&self, key: &PublicKey) -> Result<(), SignatureError> {
        let encoded = serde_json::to_string(&self.message)?;
        let signature = SshSig::from_pem(&self.signature)?;
        key.verify("builder@builds.rs", &encoded.as_bytes(), &signature)?;
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
