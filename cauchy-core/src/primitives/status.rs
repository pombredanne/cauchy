use bus::Bus;
use bytes::Bytes;
use crossbeam::channel::select;
use crossbeam::channel::Receiver;

use crypto::sketches::dummy_sketch::*;
use crypto::sketches::odd_sketch::*;
use crypto::sketches::*;
use primitives::transaction::Transaction;
use primitives::work_site::WorkSite;
use crypto::hashes::*;

use secp256k1::PublicKey;

use std::sync::RwLock;

pub struct Status {
    nonce: RwLock<u64>,
    odd_sketch: RwLock<OddSketch>,
    mini_sketch: RwLock<DummySketch>,
}

impl Status {
    pub fn new(
        nonce: RwLock<u64>,
        odd_sketch: RwLock<OddSketch>,
        mini_sketch: RwLock<DummySketch>,
    ) -> Status {
        Status {
            nonce,
            odd_sketch,
            mini_sketch,
        }
    }

    pub fn null() -> Status {
        Status {
            nonce: RwLock::new(0),
            odd_sketch: RwLock::new(OddSketch::new()),
            mini_sketch: RwLock::new(DummySketch::new()),
        }
    }

    pub fn add_to_odd_sketch<T: Identifiable>(&self, item: &T) {
        let mut sketch_locked = self.odd_sketch.write().unwrap();
        sketch_locked.insert(item);
    }

    pub fn update_odd_sketch(&self, odd_sketch: OddSketch) {
        let mut sketch_locked = self.odd_sketch.write().unwrap();
        *sketch_locked = odd_sketch;
    }

    pub fn update_mini_sketch(&self, mini_sketch: DummySketch) {
        let mut sketch_locked = self.mini_sketch.write().unwrap();
        *sketch_locked = mini_sketch;
    }

    pub fn update_nonce(&self, nonce: u64) {
        let mut nonce_locked = self.nonce.write().unwrap();
        *nonce_locked = nonce;
    }

    pub fn get_odd_sketch(&self) -> OddSketch {
        let sketch_read = self.odd_sketch.read().unwrap();
        sketch_read.clone()
    }

    pub fn get_mini_sketch(&self) -> DummySketch {
        let sketch_locked = self.mini_sketch.read().unwrap();
        sketch_locked.clone()
    }

    pub fn add_to_mini_sketch<T: Identifiable>(&self, item: &T) {
        let mut sketch_locked = self.mini_sketch.write().unwrap();
        sketch_locked.insert(item);
    }

    pub fn get_site_hash(&self, pubkey: PublicKey) -> Bytes {
        let nonce_locked = self.nonce.read().unwrap();
        let work_site = WorkSite::new(pubkey, *nonce_locked);
        work_site.get_site_hash()
    }

    pub fn get_nonce(&self) -> u64 {
        let nonce_locked = self.nonce.read().unwrap();
        *nonce_locked
    }

    pub fn update_local(
        &self,
        mut odd_sketch_bus: Bus<OddSketch>,
        tx_receive: Receiver<Transaction>,
        distance_receive: Receiver<(u64, u16)>,
    ) {
        let mut best_distance: u16 = 512;
        loop {
            select! {
                recv(tx_receive) -> tx => {
                    self.add_to_odd_sketch(&tx.clone().unwrap());
                    self.add_to_mini_sketch(&tx.unwrap());
                    odd_sketch_bus.broadcast(self.get_odd_sketch());
                    best_distance = 512;
                },
                recv(distance_receive) -> pair => {
                    let (nonce, distance) = pair.unwrap();
                    if distance < best_distance {
                        self.update_nonce(nonce);
                        best_distance = distance;
                    }
                }
            }
        }
    }
}
