use bus::Bus;
use bytes::Bytes;
use crossbeam::channel::select;
use crossbeam::channel::Receiver;

use crypto::hashes::*;
use crypto::sketches::dummy_sketch::*;
use crypto::sketches::odd_sketch::*;
use crypto::sketches::*;
use primitives::transaction::Transaction;
use primitives::work_site::WorkSite;

use utils::constants::HASH_LEN;

use secp256k1::PublicKey;

use std::sync::RwLock;

#[derive(PartialEq, Clone)]
pub struct TotalSketch {
    pub oddsketch: OddSketch,
    pub minisketch: DummySketch, // When status is professed we need not store the minisketch?
    pub root: Bytes,
}

pub struct Work {
    pub oddsketch: OddSketch,
    pub root: Bytes,
    pub nonce: u64,
}

pub struct Status {
    nonce: RwLock<u64>,
    sketch: RwLock<TotalSketch>,
}

impl Status {
    pub fn new(nonce: RwLock<u64>, sketch: RwLock<TotalSketch>) -> Status {
        Status { nonce, sketch }
    }

    pub fn null() -> Status {
        Status {
            nonce: RwLock::new(0),
            sketch: RwLock::new(TotalSketch {
                oddsketch: OddSketch::new(),
                minisketch: DummySketch::new(),
                root: Bytes::from(&[0; HASH_LEN][..]),
            }),
        }
    }

    pub fn add_item<T: Identifiable>(&self, item: &T, root: Bytes) {
        let mut sketch_locked = self.sketch.write().unwrap();
        sketch_locked.oddsketch.insert(item);
        sketch_locked.minisketch.insert(item);
        sketch_locked.root = root;
    }

    // Update oddsketch, root and minisketch
    pub fn update_total_sketch(&self, total_sketch: &TotalSketch) {
        let mut sketch_locked = self.sketch.write().unwrap();
        *sketch_locked = total_sketch.clone();
    }

    // Update oddsketch, root and nonce
    pub fn update_work(&self, work: Work) {
        let mut sketch_locked = self.sketch.write().unwrap();
        let minisketch = sketch_locked.minisketch.clone();
        *sketch_locked = TotalSketch {
            oddsketch: work.oddsketch,
            minisketch,
            root: work.root,
        };
        let mut nonce_locked = self.nonce.write().unwrap();
        *nonce_locked = work.nonce;
    }

    pub fn update_nonce(&self, nonce: u64) {
        let mut nonce_locked = self.nonce.write().unwrap();
        *nonce_locked = nonce;
    }

    pub fn get_total_sketch(&self) -> TotalSketch {
        self.sketch.read().unwrap().clone()
    }

    pub fn get_oddsketch(&self) -> OddSketch {
        let sketch_read = self.sketch.read().unwrap();
        sketch_read.oddsketch.clone()
    }

    pub fn get_minisketch(&self) -> DummySketch {
        let sketch_locked = self.sketch.read().unwrap();
        sketch_locked.minisketch.clone()
    }

    pub fn get_site_hash(&self, pubkey: PublicKey) -> Bytes {
        let nonce_read = *self.nonce.read().unwrap();
        let sketch_read = self.sketch.read().unwrap();
        let work_site = WorkSite::new(pubkey, sketch_read.root.clone(), nonce_read);
        work_site.get_site_hash()
    }

    pub fn get_nonce(&self) -> u64 {
        let nonce_locked = self.nonce.read().unwrap();
        *nonce_locked
    }

    pub fn update_local(
        &self,
        mut odd_sketch_bus: Bus<(OddSketch, Bytes)>,
        tx_receive: Receiver<Transaction>,
        distance_receive: Receiver<(u64, u16)>,
    ) {
        let mut best_distance: u16 = 512;
        loop {
            select! {
                recv(tx_receive) -> tx => {
                    let root = Bytes::from(&[0; 32][..]); // TODO: Actually get root
                    // TODO: These should be simulatenously locked/unlocked
                    self.add_item(&tx.unwrap(), root.clone());
                    odd_sketch_bus.broadcast((self.get_oddsketch(), root));
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
