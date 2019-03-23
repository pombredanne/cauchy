use bytes::Bytes;
use failure::Error;
use secp256k1::key::{PublicKey, SecretKey, ONE_KEY};
use secp256k1::rand::OsRng;
use secp256k1::{Message, Secp256k1, Signature};

use crypto::hashes::blake2b::*;
use utils::errors::{InvalidPubkey, InvalidSignature};

pub fn generate_keypair() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    let mut rng = OsRng::new().expect("OsRng");
    secp.generate_keypair(&mut rng)
}

pub fn generate_dummy_pubkey() -> PublicKey {
    let secp = Secp256k1::new();
    PublicKey::from_secret_key(&secp, &ONE_KEY)
}

pub fn message_from_preimage<T>(raw: T) -> Message
where
    T: Blk2bHashable,
{
    Message::from_slice(&raw.blake2b()[..32]).expect("32 bytes")
}

pub fn bytes_from_pubkey(key: PublicKey) -> Bytes {
    Bytes::from(&key.serialize()[..])
}

pub fn pubkey_from_bytes(raw: Bytes) -> Result<PublicKey, Error> {
    match PublicKey::from_slice(&raw) {
        Ok(some) => Ok(some),
        Err(_) => Err(InvalidPubkey.into()),
    }
}

pub fn bytes_from_sig(sig: Signature) -> Bytes {
    Bytes::from(&sig.serialize_compact()[..])
}

pub fn sig_from_bytes(raw: Bytes) -> Result<Signature, Error> {
    match Signature::from_compact(&raw) {
        Ok(sig) => Ok(sig),
        Err(_) => Err(InvalidSignature.into()),
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
