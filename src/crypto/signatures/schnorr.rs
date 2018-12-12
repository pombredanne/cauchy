use bytes::Bytes;
use crypto::hashes::blake2b::*;
use secp256k1::key::{PublicKey, SecretKey};
use secp256k1::rand::OsRng;
use secp256k1::{Message, Secp256k1, Signature};

pub fn generate_keypair() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    let mut rng = OsRng::new().expect("OsRng");
    secp.generate_keypair(&mut rng)
}

pub fn message_from_preimage<T>(raw: T) -> Message
where
    T: Blk2bHashable,
{
    Message::from_slice(&raw.blake2b()[..32]).expect("32 bytes")
}

pub fn pubkey_to_bytes(key: PublicKey) -> Bytes {
    Bytes::from(&key.serialize()[..])
}

pub fn bytes_to_pubkey(raw: Bytes) -> Result<PublicKey, String> {
    match PublicKey::from_slice(&raw) {
        Ok(some) => Ok(some),
        Err(_) => Err("Incorrect pubkey format".to_string()),
    }
}

pub fn sign(msg: &Message, sk: &SecretKey) -> Signature {
    let secp = Secp256k1::signing_only();
    secp.sign(msg, sk)
}

pub fn verify(msg: &Message, sig: &Signature, pk: &PublicKey) -> Result<bool, String> {
    let secp = Secp256k1::verification_only();
    match secp.verify(msg, sig, pk) {
        Ok(()) => Ok(true),
        Err(secp256k1::Error::IncorrectSignature) => Ok(false),
        Err(secp256k1::Error::InvalidPublicKey) => Err("Invalid Pubkey".to_string()),
        Err(secp256k1::Error::InvalidSignature) => Err("Invalid Signature".to_string()),
        Err(_) => Err("Invalid Message".to_string()),
    }
}