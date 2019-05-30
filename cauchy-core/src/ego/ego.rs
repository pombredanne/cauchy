use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use bus::BusReader;
use bytes::Bytes;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{Future, Sink};
use log::info;
use rand::Rng;
use secp256k1::{PublicKey, SecretKey, Signature};

use crate::{
    crypto::{
        hashes::Identifiable,
        signatures::ecdsa,
        sketches::{dummy_sketch::DummySketch, odd_sketch::OddSketch, SketchInsertable},
    },
    net::messages::*,
    primitives::{
        status::Status,
        transaction::Transaction,
        varint::VarInt,
        work::{WorkSite, WorkStack, WorkState},
    },
    utils::constants::{CONFIG, HASH_LEN},
};

pub struct Ego {
    pubkey: PublicKey,
    pub seckey: SecretKey,

    pub work_stack: WorkStack,
    pub minisketch: DummySketch,
    pub current_distance: u16,
}

impl Ego {
    pub fn new(pubkey: PublicKey, seckey: SecretKey) -> Ego {
        Ego {
            pubkey,
            seckey,
            work_stack: Default::default(),
            current_distance: 512,
            minisketch: Default::default(),
        }
    }

    pub fn generate_end_handshake(&self, secret: u64) -> Message {
        Message::EndHandshake {
            pubkey: self.pubkey,
            sig: ecdsa::sign(
                &ecdsa::message_from_preimage(Bytes::from(VarInt::new(secret))),
                &self.seckey,
            ),
        }
    }

    pub fn get_work_site(&self) -> WorkSite {
        WorkSite::new(
            self.pubkey,
            self.work_stack.get_root(),
            self.work_stack.get_nonce(),
        )
    }

    pub fn get_pubkey(&self) -> PublicKey {
        self.pubkey
    }

    pub fn update_current_distance(&mut self, new_distance: u16) {
        self.current_distance = new_distance;
    }

    pub fn get_current_distance(&self) -> u16 {
        self.current_distance
    }

    pub fn update_minisketch(&mut self, minisketch: DummySketch) {
        self.minisketch = minisketch;
    }

    pub fn increment(&mut self, new_tx: &Transaction, new_root: Bytes) {
        self.work_stack.update(new_tx, new_root);
        self.minisketch.insert(new_tx);
    }

    pub fn pull(&mut self, oddsketch: OddSketch, minisketch: DummySketch, root: Bytes) {
        self.work_stack.update_oddsketch(oddsketch);
        self.work_stack.update_root(root);
        self.work_stack.update_nonce(0);
        self.minisketch = minisketch;
    }

    pub fn get_work_stack(&self) -> WorkStack {
        self.work_stack.clone()
    }

    pub fn get_minisketch(&self) -> DummySketch {
        self.minisketch.clone()
    }

    // Mining updates
    pub fn updater(
        ego: Arc<Mutex<Ego>>,
        distance_receive: std::sync::mpsc::Receiver<(u64, u16)>,
        mut mining_reset: BusReader<(OddSketch, Bytes)>,
    ) {
        let mut best_distance: u16 = 512;

        loop {
            if let Ok((nonce, distance)) = distance_receive.recv() {
                if mining_reset.try_recv().is_ok() {
                    let mut ego_locked = ego.lock().unwrap();
                    ego_locked.work_stack.update_nonce(nonce);
                    ego_locked.update_current_distance(best_distance);
                    best_distance = distance;
                } else if distance < best_distance {
                    let mut ego_locked = ego.lock().unwrap();
                    ego_locked.work_stack.update_nonce(nonce);
                    ego_locked.update_current_distance(best_distance);
                    best_distance = distance;
                }
            }
        }
    }
}
