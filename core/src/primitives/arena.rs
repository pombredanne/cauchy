use bytes::Bytes;
use primitives::status::{StaticStatus, Status};
use secp256k1::PublicKey;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use utils::byte_ops::Hamming;

pub struct Arena {
    // TODO: Individual RWLocks for each? Asyncronous hasmaps?
    // Is it worth it given the time between arena lookups is long
    // Probably not?
    self_pubkey: PublicKey,
    // TODO: Combine these?
    peer_status: HashMap<PublicKey, Arc<Status>>,
    perceived_status: HashMap<PublicKey, StaticStatus>,
    order: Vec<PublicKey>,
}

impl Arena {
    pub fn init(self_pubkey: &PublicKey, self_status: Arc<Status>) -> Arena {
        let mut new = Arena {
            self_pubkey: *self_pubkey,
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

        let value = self.perceived_status.remove(pubkey_a).unwrap();
        self.perceived_status.insert(*pubkey_b, value);
    }

    pub fn new_peer(&mut self, pubkey: &PublicKey) {
        self.add_peer(pubkey, Arc::new(Status::null()))
    }

    pub fn add_peer(&mut self, pubkey: &PublicKey, status: Arc<Status>) {
        println!("Added new peer to arena!");
        self.peer_status.insert(*pubkey, status);
        self.perceived_status.insert(*pubkey, StaticStatus::null()); // Remove for self
    }

    pub fn get_perception(&self, pubkey: &PublicKey) -> Option<&StaticStatus> {
        self.perceived_status.get(pubkey)
    }

    pub fn update_perception(&mut self, pubkey: &PublicKey) {
        self.perceived_status.insert(
            *pubkey,
            self.peer_status.get(&self.self_pubkey).unwrap().to_static(),
        );
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

        println!("Distances: {:?}", distances.values());
        let mut ordered: Vec<PublicKey> = self.peer_status.keys().map(|key| *key).collect();
        ordered.sort_by_key(|x| distances.get(x));
        println!("Order: {:?}", ordered);
        self.order = ordered
    }

    pub fn get_order(&self) -> Vec<PublicKey> {
        self.order.clone()
    }
}
