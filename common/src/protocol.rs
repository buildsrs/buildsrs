use ssh_key::{HashAlg, PrivateKey};
use bytes::Bytes;

pub enum ServerMessage {
    AuthRequest(Bytes),
}

pub enum ClientMessage {
    AuthResponse(Bytes),
}

fn sign(key: &PrivateKey, data: &[u8]) -> Vec<u8> {
    key.sign("builder@builds.rs", HashAlg::Sha512, data)
        .unwrap()
        .signature_bytes()
        .to_vec()
}
