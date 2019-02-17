use bytes::Bytes;
use primitives::status::Status;
use secp256k1::PublicKey;
use std::collections::HashMap;
use std::sync::Arc;
use utils::byte_ops::Hamming;
use utils::constants::ARENA_VERBOSE;

pub struct Arena {
    // TODO: Individual RWLocks for each? Asyncronous hasmaps?
    // Is it worth it given the time between arena lookups is long
    // Probably not?
    local_pubkey: PublicKey,
    // TODO: Combine these?
    peer_status: HashMap<PublicKey, Arc<Status>>,
    perceived_status: HashMap<PublicKey, Arc<Status>>,
    order: Vec<PublicKey>,
}

impl Arena {
    pub fn init(self_pubkey: &PublicKey, self_status: Arc<Status>) -> Arena {
        let mut new = Arena {
            local_pubkey: *self_pubkey,
            peer_status: HashMap::new(),
            perceived_status: HashMap::new(),
            order: Vec::new(),
        };
        new.add_peer(self_pubkey, self_status);
        new
    }

    pub fn replace_key(&mut self, pubkey_a: &PublicKey, pubkey_b: &PublicKey) {
        let value = self.peer_status.remove(pubkey_a).unwrap();
        self.peer_status.insert(*pubkey_b, value);
        if *pubkey_a != self.local_pubkey {
            let value = self.perceived_status.remove(pubkey_a).unwrap();
            self.perceived_status.insert(*pubkey_b, value);
        }
    }

    pub fn new_peer(&mut self, pubkey: &PublicKey) {
        // TOOD: Catch adding own local key?
        self.add_peer(pubkey, Arc::new(Status::null()))
    }

    pub fn add_peer(&mut self, pubkey: &PublicKey, status: Arc<Status>) {
        if ARENA_VERBOSE {
            println!("Added new peer to arena!");
        }
        self.peer_status.insert(*pubkey, status);
        if *pubkey != self.local_pubkey {
            self.perceived_status
                .insert(*pubkey, Arc::new(Status::null()));
        }
    }

    pub fn get_perception(&self, pubkey: &PublicKey) -> Option<Arc<Status>> {
        let status = self.perceived_status.get(pubkey)?;
        Some(status.clone())
    }

    pub fn get_status(&self, pubkey: &PublicKey) -> Option<Arc<Status>> {
        let status = self.peer_status.get(pubkey)?;
        Some(status.clone())
    }

    pub fn update_order(&mut self) {
        let site_hashes: Vec<Bytes> = self
            .peer_status
            .iter()
            .map(|(pubkey, status)| status.get_site_hash(*pubkey))
            .collect();

        let distances: HashMap<PublicKey, u16> = self
            .peer_status
            .iter()
            .map(|(pubkey, status)| {
                let mut dist: u16 = 0;
                for site_hash in site_hashes.iter().by_ref() {
                    dist += Bytes::hamming_distance(&status.get_odd_sketch(), site_hash);
                }
                (*pubkey, dist)
            })
            .collect();

        let mut ordered: Vec<PublicKey> = self.peer_status.keys().cloned().collect();
        ordered.sort_by_key(|x| {
            let d = &distances[x];
            if ARENA_VERBOSE {
                println!("Key: {} \n Distance: {}", x, d)
            }
            d
        });
        self.order = ordered
    }

    pub fn get_order(&self) -> Vec<PublicKey> {
        self.order.clone()
    }
}
