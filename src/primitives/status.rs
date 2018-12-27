use bus::BusReader;
use bytes::Bytes;
use primitives::work_site::WorkSite;
use secp256k1::PublicKey;
use std::sync::mpsc::Receiver;
use std::sync::RwLock;
use utils::constants::SKETCH_LEN;

pub struct StaticStatus {
    pub nonce: u64,
    pub state_sketch: Bytes,
}

impl StaticStatus {
    pub fn null() -> StaticStatus {
        StaticStatus {
            nonce: 0,
            state_sketch: Bytes::from(&[0; SKETCH_LEN][..]),
        }
    }
}

pub struct Status {
    nonce: RwLock<u64>,
    state_sketch: RwLock<Bytes>,
}

impl Status {
    pub fn new(nonce: RwLock<u64>, state_sketch: RwLock<Bytes>) -> Status {
        Status {
            nonce,
            state_sketch,
        }
    }

    pub fn to_static(&self) -> StaticStatus {
        StaticStatus {
            nonce: self.get_nonce(),
            state_sketch: self.get_state_sketch(),
        }
    }

    pub fn null() -> Status {
        Status {
            nonce: RwLock::new(0),
            state_sketch: RwLock::new(Bytes::from(&[0; SKETCH_LEN][..])),
        }
    }

    pub fn update_state_sketch(&self, sketch: Bytes) {
        let mut sketch_locked = self.state_sketch.write().unwrap();
        *sketch_locked = sketch;
    }

    pub fn update_nonce(&self, nonce: u64) {
        println!("Updated nonce!");
        let mut nonce_locked = self.nonce.write().unwrap();
        *nonce_locked = nonce;
    }

    pub fn get_state_sketch(&self) -> Bytes {
        let sketch_locked = self.state_sketch.read().unwrap();
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
                    self.update_state_sketch(sketch);
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
