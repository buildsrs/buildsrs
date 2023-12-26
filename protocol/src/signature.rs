//! Wrapper type for signed messages

use serde::{Deserialize, Serialize};
use ssh_key::{HashAlg, PrivateKey, PublicKey, SshSig};

/// Signature namespace for verified messages
const NAMESPACE_BUILDSRS: &str = "builder@builds.rs";

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
