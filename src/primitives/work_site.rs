use bytes::Bytes;
use crypto::hashes::blake2b::Blk2bHashable;
use secp256k1::PublicKey;
use std::sync::{Arc, Mutex};
use utils::byte_ops::Hamming;
use utils::constants::TX_ID_LEN;

#[derive(Debug, Clone)]
pub struct WorkSite {
    public_key: PublicKey,
    nonce: Arc<Mutex<u64>>,
}

impl WorkSite {
    pub fn init(pk: PublicKey) -> WorkSite {
        WorkSite {
            public_key: pk,
            nonce: Arc::new(Mutex::new(0)),
        }
    }

    pub fn new(pk: PublicKey, nonce: u64) -> WorkSite {
        WorkSite {
            public_key: pk,
            nonce: Arc::new(Mutex::new(nonce)),
        }
    }

    pub fn increment(&self) {
        let mut nonce_locked = self.nonce.lock().unwrap();
        *nonce_locked += 1;
    }

    pub fn set_nonce(&self, nonce: u64) {
        let mut nonce_locked = self.nonce.lock().unwrap();
        *nonce_locked += nonce;    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn get_nonce(&self) -> u64 {
        let nonce_locked = self.nonce.lock().unwrap();
        (*nonce_locked).clone()
    }

    pub fn get_site_hash(&self) -> Bytes {
        Bytes::from(&self.blake2b()[..TX_ID_LEN])
    }

    pub fn mine(&self, state_sketch: &Bytes) -> u32 {
        self.get_site_hash().hamming_distance(state_sketch.clone())
    }
}
