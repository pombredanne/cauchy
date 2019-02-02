use bus::Bus;
use bytes::{Bytes, BytesMut};
use crossbeam::channel::select;
use crossbeam::channel::Receiver;

use crypto::hashes::blake2b::Blk2bHashable;
use crypto::sketches::iblt::*;
use crypto::sketches::odd_sketch::*;
use primitives::transaction::Transaction;
use primitives::work_site::WorkSite;
use utils::constants::*;

use secp256k1::PublicKey;

use std::sync::RwLock;

#[derive(Clone)]
pub struct StaticStatus {
    pub nonce: u64,
    pub odd_sketch: Bytes,
    pub mini_sketch: IBLT,
}

impl StaticStatus {
    pub fn null() -> StaticStatus {
        StaticStatus {
            nonce: 0,
            odd_sketch: Bytes::from(&[0; SKETCH_CAPACITY][..]),
            mini_sketch: IBLT::with_capacity(SKETCH_CAPACITY, IBLT_N_HASHES),
        }
    }
}

pub struct Status {
    nonce: RwLock<u64>,
    odd_sketch: RwLock<BytesMut>,
    mini_sketch: RwLock<IBLT>,
}

impl Status {
    pub fn new(
        nonce: RwLock<u64>,
        odd_sketch: RwLock<BytesMut>,
        mini_sketch: RwLock<IBLT>,
    ) -> Status {
        Status {
            nonce,
            odd_sketch,
            mini_sketch,
        }
    }

    pub fn to_static(&self) -> StaticStatus {
        StaticStatus {
            nonce: self.get_nonce(),
            odd_sketch: self.get_odd_sketch(),
            mini_sketch: self.get_mini_sketch(),
        }
    }

    pub fn null() -> Status {
        Status {
            nonce: RwLock::new(0),
            odd_sketch: RwLock::new(BytesMut::from(&[0; SKETCH_CAPACITY][..])),
            mini_sketch: RwLock::new(IBLT::with_capacity(SKETCH_CAPACITY, IBLT_N_HASHES)),
        }
    }

    pub fn add_to_odd_sketch<T: Blk2bHashable>(&self, item: &T) {
        let mut sketch_locked = self.odd_sketch.write().unwrap();
        add_to_bin(&mut *sketch_locked, item);
    }

    pub fn update_odd_sketch(&self, mini_sketch: Bytes) {
        let mut sketch_locked = self.odd_sketch.write().unwrap();
        *sketch_locked = BytesMut::from(mini_sketch);
    }

    pub fn update_mini_sketch(&self, mini_sketch: IBLT) {
        let mut sketch_locked = self.mini_sketch.write().unwrap();
        *sketch_locked = mini_sketch;
    }

    pub fn update_nonce(&self, nonce: u64) {
        let mut nonce_locked = self.nonce.write().unwrap();
        *nonce_locked = nonce;
    }

    pub fn get_odd_sketch(&self) -> Bytes {
        let sketch_locked = self.odd_sketch.read().unwrap();
        (*sketch_locked).clone().freeze()
    }

    pub fn get_mini_sketch(&self) -> IBLT {
        let sketch_locked = self.mini_sketch.read().unwrap();
        (*sketch_locked).clone()
    }

    pub fn add_to_mini_sketch<T: Blk2bHashable>(&self, item: &T) {
        let mut sketch_locked = self.mini_sketch.write().unwrap();
        (*sketch_locked).insert(item.blake2b());
    }

    pub fn get_site_hash(&self, pubkey: PublicKey) -> Bytes {
        let nonce_locked = self.nonce.read().unwrap();
        let work_site = WorkSite::new(pubkey, *nonce_locked);
        work_site.get_site_hash()
    }

    pub fn get_nonce(&self) -> u64 {
        let nonce_locked = self.nonce.read().unwrap();
        (*nonce_locked).clone()
    }

    pub fn update_local(
        &self,
        mut odd_sketch_bus: Bus<Bytes>,
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
                    //println!("Updated mini_sketch");
                },
                recv(distance_receive) -> pair => {
                    let (nonce, distance) = pair.unwrap();
                    if distance < best_distance {
                        self.update_nonce(nonce);
                        best_distance = distance;
                        //println!("Updated nonce: {}", nonce);
                    }
                }
            }
        }
    }
}
