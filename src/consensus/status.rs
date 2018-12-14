use bytes::Bytes;
use crypto::hashes::blake2b::*;
use primitives::work_site::*;
use secp256k1::PublicKey;
use std::sync::{Arc, Mutex};
use utils::timing::*;

// TODO: Async
pub struct Status {
    public_key: PublicKey,
    nonce: Arc<Mutex<u64>>,

    last_state_update: Arc<Mutex<u64>>,
    state_sketch: Arc<Mutex<Bytes>>,

    last_site_update: Arc<Mutex<u64>>,

    digested: Arc<Mutex<bool>>,
    site_hash: Arc<Mutex<Bytes>>,
}

impl Status {
    pub fn new(state_sketch: Bytes, work_site: &WorkSite) -> Status {
        Status {
            public_key: work_site.get_public_key(),
            nonce: Arc::new(Mutex::new(work_site.get_nonce())),
            last_state_update: Arc::new(Mutex::new(get_current_time())),
            state_sketch: Arc::new(Mutex::new(state_sketch)),
            last_site_update: Arc::new(Mutex::new(get_current_time())),
            site_hash: Arc::new(Mutex::new(work_site.blake2b())),
            digested: Arc::new(Mutex::new(true)),
        }
    }

    pub fn update_site_hash(&self) {
        let mut digested_locked = self.digested.lock().unwrap();
        if !*digested_locked {
            let mut site_hash_locked = self.site_hash.lock().unwrap();
            *site_hash_locked = WorkSite::new(self.get_public_key(), self.get_nonce()).blake2b();
            *digested_locked = true;
        }
    }

    pub fn update_state_sketch(&self, sketch: Bytes) {
        let mut sketch_locked = self.state_sketch.lock().unwrap();
        *sketch_locked = sketch;
        let mut last_state_locked = self.last_state_update.lock().unwrap();
        *last_state_locked = get_current_time();
    }

    pub fn update_nonce(&self, nonce: u64) {
        let mut digested_unlocked = self.digested.lock().unwrap();
        *digested_unlocked = false;

        let mut nonce_locked = self.nonce.lock().unwrap();
        *nonce_locked = nonce;

        let mut last_site_unlocked = self.last_site_update.lock().unwrap();
        *last_site_unlocked = get_current_time();
    }

    pub fn get_state_sketch(&self) -> Bytes {
        let sketch_locked = self.state_sketch.lock().unwrap();
        (*sketch_locked).clone()
    }

    pub fn get_site_hash(&self) -> Bytes {
        self.update_site_hash();
        let site_hash_locked = self.site_hash.lock().unwrap();
        (*site_hash_locked).clone()
    }

    pub fn get_nonce(&self) -> u64 {
        let nonce_locked = self.nonce.lock().unwrap();
        (*nonce_locked).clone()
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key
    }
}
