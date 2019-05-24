pub mod ego;
pub mod peer_ego;

use bytes::Bytes;

use crate::crypto::sketches::{dummy_sketch::DummySketch, odd_sketch::OddSketch};

pub trait WorkState {
    fn get_oddsketch(&self) -> OddSketch;
    fn get_root(&self) -> Bytes;
    fn get_nonce(&self) -> u64;
    fn update_oddsketch(&mut self, oddsketch: OddSketch);
    fn update_root(&mut self, root: Bytes);
    fn update_nonce(&mut self, nonce: u64);
}

#[derive(Clone)]
pub struct WorkStack {
    root: Bytes,
    nonce: u64,
    oddsketch: OddSketch,
    minisketch: DummySketch, // The minisketch to send to peer
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
