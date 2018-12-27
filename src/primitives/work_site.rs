use bytes::Bytes;
use crypto::hashes::blake2b::Blk2bHashable;
use secp256k1::PublicKey;
use std::cell::Cell;
use utils::byte_ops::Hamming;
use utils::constants::TX_ID_LEN;

#[derive(Debug, Clone)]
pub struct WorkSite {
    public_key: PublicKey,
    nonce: Cell<u64>,
}

impl WorkSite {
    pub fn new(pk: PublicKey, nonce: u64) -> WorkSite {
        WorkSite {
            public_key: pk,
            nonce: Cell::new(nonce),
        }
    }

    pub fn increment(&self) {
        self.nonce.set(self.nonce.get() + 1);
    }

    pub fn set_nonce(&self, nonce: u64) {
        self.nonce.set(nonce);
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn get_nonce(&self) -> u64 {
        self.nonce.get()
    }

    pub fn get_site_hash(&self) -> Bytes {
        Bytes::from(&self.blake2b().blake2b()[..TX_ID_LEN])
    }

    pub fn mine(&self, state_sketch: &Bytes) -> u16 {
        Bytes::hamming_distance(&self.get_site_hash(), state_sketch)
    }
}
