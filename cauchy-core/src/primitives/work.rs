use bytes::Bytes;
use secp256k1::PublicKey;

use crate::{
    crypto::{
        hashes::blake2b::Blk2bHashable,
        sketches::{odd_sketch::*, SketchInsertable},
    },
    primitives::transaction::Transaction,
    utils::{byte_ops::Hamming, constants::HASH_LEN},
};

pub trait WorkState {
    fn get_oddsketch(&self) -> OddSketch;
    fn get_root(&self) -> Bytes;
    fn get_nonce(&self) -> u64;
    fn update_oddsketch(&mut self, oddsketch: OddSketch);
    fn update_root(&mut self, root: Bytes);
    fn update_nonce(&mut self, nonce: u64);
}

#[derive(Clone, PartialEq)]
pub struct WorkStack {
    root: Bytes,
    nonce: u64,
    oddsketch: OddSketch,
}

impl Default for WorkStack {
    fn default() -> WorkStack {
        WorkStack {
            root: Bytes::from(&[0; HASH_LEN][..]),
            nonce: 0,
            oddsketch: Default::default(),
        }
    }
}

impl WorkStack {
    pub fn new(root: Bytes, oddsketch: OddSketch, nonce: u64) -> WorkStack {
        WorkStack {
            root,
            oddsketch,
            nonce,
        }
    }
    pub fn update(&mut self, new_tx: &Transaction, new_root: Bytes) {
        self.nonce = 0;
        self.oddsketch.insert(new_tx);
        self.root = new_root;
    }
}

impl WorkState for WorkStack {
    fn get_oddsketch(&self) -> OddSketch {
        self.oddsketch.clone()
    }
    fn get_root(&self) -> Bytes {
        self.root.clone()
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn update_oddsketch(&mut self, oddsketch: OddSketch) {
        self.oddsketch = oddsketch;
    }
    fn update_root(&mut self, root: Bytes) {
        self.root = root;
    }
    fn update_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
    }
}

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

    pub fn mine(&self, oddsketch: OddSketch) -> u16 {
        Bytes::hamming_distance(&self.get_site_hash(), &Bytes::from(oddsketch)) // TODO: Clunky, fix
    }
}
