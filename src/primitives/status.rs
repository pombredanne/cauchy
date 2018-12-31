use bus::BusReader;
use bytes::Bytes;
use primitives::work_site::WorkSite;
use secp256k1::PublicKey;
use std::sync::mpsc::Receiver;
use std::sync::RwLock;
use utils::constants::SKETCH_LEN;

pub struct StaticStatus {
    pub nonce: u64,
    pub odd_sketch: Bytes,
    pub sketch: Bytes,
}

impl StaticStatus {
    pub fn null() -> StaticStatus {
        StaticStatus {
            nonce: 0,
            odd_sketch: Bytes::from(&[0; SKETCH_LEN][..]),
            sketch: Bytes::new(),
        }
    }
}

pub struct Status {
    nonce: RwLock<u64>,
    odd_sketch: RwLock<Bytes>,
    sketch: RwLock<Bytes>,
}

impl Status {
    pub fn new(nonce: RwLock<u64>, odd_sketch: RwLock<Bytes>, sketch: RwLock<Bytes>) -> Status {
        Status {
            nonce,
            odd_sketch,
            sketch,
        }
    }

    pub fn to_static(&self) -> StaticStatus {
        StaticStatus {
            nonce: self.get_nonce(),
            odd_sketch: self.get_odd_sketch(),
            sketch: self.get_sketch(),
        }
    }

    pub fn null() -> Status {
        Status {
            nonce: RwLock::new(0),
            odd_sketch: RwLock::new(Bytes::from(&[0; SKETCH_LEN][..])),
            sketch: RwLock::new(Bytes::new()),
        }
    }

    pub fn update_odd_sketch(&self, sketch: Bytes) {
        println!("Updated state sketch!");
        let mut sketch_locked = self.odd_sketch.write().unwrap();
        *sketch_locked = sketch;
    }

    pub fn update_sketch(&self, sketch: Bytes) {
        println!("Updated sketch!");
        let mut sketch_locked = self.sketch.write().unwrap();
        *sketch_locked = sketch;
    }

    pub fn update_nonce(&self, nonce: u64) {
        println!("Updated nonce!");
        let mut nonce_locked = self.nonce.write().unwrap();
        *nonce_locked = nonce;
    }

    pub fn get_odd_sketch(&self) -> Bytes {
        let sketch_locked = self.odd_sketch.read().unwrap();
        (*sketch_locked).clone()
    }

    pub fn get_sketch(&self) -> Bytes {
        let sketch_locked = self.sketch.read().unwrap();
        (*sketch_locked).clone()
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
        mut sketch_receive: BusReader<Bytes>,
        distance_receive: Receiver<(u64, u16)>,
    ) {
        let mut best_distance: u16 = 512;
        loop {
            match sketch_receive.try_recv() {
                Ok(sketch) => {
                    self.update_odd_sketch(sketch);
                    best_distance = 512;
                }
                Err(_) => (),
            }
            match distance_receive.try_recv() {
                Ok((nonce, distance)) => {
                    if distance < best_distance {
                        self.update_nonce(nonce);
                        best_distance = distance;
                    }
                }
                Err(_) => (),
            }
        }
    }
}
