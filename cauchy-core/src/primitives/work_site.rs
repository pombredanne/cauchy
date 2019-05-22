use bytes::Bytes;
use secp256k1::PublicKey;

use crate::{
    crypto::{hashes::blake2b::Blk2bHashable, sketches::odd_sketch::*},
    utils::byte_ops::Hamming,
};

#[derive(Debug, Clone)]
pub struct WorkSite {
    public_key: PublicKey,
    root: Bytes,
    nonce: u64,
}

impl WorkSite {
    pub fn new(public_key: PublicKey, root: Bytes, nonce: u64) -> WorkSite {
        WorkSite {
            public_key,
            root,
            nonce,
        }
    }

    pub fn increment(&mut self) {
        self.nonce += 1;
    }

    pub fn set_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn get_root(&self) -> Bytes {
        self.root.clone()
    }

    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }

    pub fn get_site_hash(&self) -> Bytes {
        self.blake2b().blake2b()
    }

    pub fn mine(&self, state_sketch: &OddSketch) -> u16 {
        Bytes::hamming_distance(&self.get_site_hash(), &Bytes::from(state_sketch.clone())) // TODO: Clunky, fix
    }
}
