use bytes::Bytes;
use primitives::status::{StaticStatus, Status};
use secp256k1::PublicKey;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use utils::byte_ops::Hamming;
use utils::constants::*;

pub struct Arena {
    self_pubkey: PublicKey,
    live_peers: HashSet<PublicKey>,
    peer_status: HashMap<PublicKey, Arc<Status>>,
    perceived_status: HashMap<PublicKey, StaticStatus>,
    order: Vec<PublicKey>,
}

impl Arena {
    pub fn new(self_pubkey: &PublicKey, self_status: Arc<Status>) -> Arena {
        let mut new = Arena {
            self_pubkey: *self_pubkey,
            live_peers: HashSet::new(),
            peer_status: HashMap::new(),
            perceived_status: HashMap::new(),
            order: Vec::new(),
        };
        new.add_peer(self_pubkey, self_status);
        new
    }

    pub fn add_peer(&mut self, pubkey: &PublicKey, status: Arc<Status>) {
        println!("Added new peer to arena!");
        self.peer_status.insert(*pubkey, status);
        self.perceived_status.insert(*pubkey, StaticStatus::null()); // Remove for self
        self.live_peers.insert(*pubkey);
    }

    pub fn get_perception(&self, pubkey: &PublicKey) -> StaticStatus {
        (*self.perceived_status.get(pubkey).unwrap()).clone()
    }

    pub fn update_perception(&mut self, pubkey: &PublicKey) {
        self.perceived_status.insert(
            *pubkey,
            self.peer_status.get(&self.self_pubkey).unwrap().to_static(),
        );
    }

    pub fn get_peer(&self, pubkey: &PublicKey) -> Arc<Status> {
        (*self.peer_status.get(pubkey).unwrap()).clone()
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

        let mut ordered: Vec<PublicKey> = self.live_peers.clone().into_iter().collect();
        ordered.sort_by_key(|x| distances.get(x));
        self.order = ordered
    }
}
