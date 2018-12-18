use bus::BusReader;
use bytes::Bytes;
use secp256k1::PublicKey;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Status {
    public_key: PublicKey,
    nonce: Arc<Mutex<u64>>,
    state_sketch: Arc<Mutex<Bytes>>,
}

impl Status {
    pub fn new(
        public_key: PublicKey,
        nonce: Arc<Mutex<u64>>,
        state_sketch: Arc<Mutex<Bytes>>,
    ) -> Self {
        Status {
            public_key,
            nonce,
            state_sketch,
        }
    }

    pub fn update_state_sketch(&self, sketch: Bytes) {
        let mut sketch_locked = self.state_sketch.lock().unwrap();
        *sketch_locked = sketch;
    }

    pub fn update_nonce(&self, nonce: u64) {
        let mut nonce_locked = self.nonce.lock().unwrap();
        *nonce_locked = nonce;
    }

    pub fn get_state_sketch(&self) -> Bytes {
        let sketch_locked = self.state_sketch.lock().unwrap();
        (*sketch_locked).clone()
    }

    pub fn get_nonce(&self) -> u64 {
        let nonce_locked = self.nonce.lock().unwrap();
        (*nonce_locked).clone()
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key
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
