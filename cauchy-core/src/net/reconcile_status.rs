use bytes::Bytes;
use crypto::hashes::*;
use crypto::signatures::ecdsa::generate_dummy_pubkey;
use primitives::status::*;
use primitives::transaction::*;
use secp256k1::PublicKey;
use std::collections::HashSet;
use std::sync::Arc;
use utils::timing::*;

use crypto::sketches::dummy_sketch::*;
use crypto::sketches::odd_sketch::*;
use crypto::sketches::*;

pub struct ReconciliationStatus {
    live: bool,
    target: PublicKey,
    start_time: u64,
    missing_ids: HashSet<Bytes>,
    excess_ids: HashSet<Bytes>,
    reconcilees: HashSet<PublicKey>,
}

impl ReconciliationStatus {
    pub fn new() -> ReconciliationStatus {
        let dummy_pk = generate_dummy_pubkey();
        ReconciliationStatus {
            start_time: 0,
            live: false,
            target: dummy_pk,
            missing_ids: HashSet::new(),
            excess_ids: HashSet::new(),
            reconcilees: HashSet::new(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.live
    }

    pub fn stop(&mut self) {
        self.live = false;
    }

    pub fn get_start_time(&self) -> u64 {
        self.start_time
    }

    pub fn set_target(&mut self, new_target: &PublicKey) {
        self.start_time = get_current_time();
        self.live = true;
        self.target = *new_target;
    }

    pub fn target_eq(&self, other: &PublicKey) -> bool {
        self.target == *other && self.live
    }

    pub fn set_ids(&mut self, excess_ids: &HashSet<Bytes>, missing_ids: &HashSet<Bytes>) {
        self.excess_ids = excess_ids.clone();
        self.missing_ids = missing_ids.clone();
    }

    pub fn get_tx_ids(&self) -> (&HashSet<Bytes>, &HashSet<Bytes>) {
        (&self.excess_ids, &self.missing_ids)
    }

    pub fn final_update(&self, local_status: Arc<Status>, perception: Arc<Status>) {
        // TODO: Revamp all of this
        let perceived_oddsketch = perception.get_oddsketch();
        let perceived_minisketch = perception.get_minisketch();

        let root = Bytes::from(&[0; 32][..]);

        let mut minisketch = perceived_minisketch
            - DummySketch::from((self.excess_ids.clone(), self.missing_ids.clone()));
        minisketch.collect();
        let oddsketch = perceived_oddsketch
            .xor(&OddSketch::sketch_ids(&self.excess_ids))
            .xor(&OddSketch::sketch_ids(&self.missing_ids));

        local_status.update_all_sketch(&AllSketch {
            minisketch,
            oddsketch,
            root,
        })
    }

    pub fn missing_ids_eq(&self, other: &HashSet<Transaction>) -> bool {
        let received_tx_ids: HashSet<Bytes> = other.iter().map(move |tx| tx.get_id()).collect();
        self.missing_ids == received_tx_ids
    }

    pub fn add_reconcilee(&mut self, pubkey: &PublicKey) {
        self.reconcilees.insert(*pubkey);
    }

    pub fn remove_reconcilee(&mut self, pubkey: &PublicKey) {
        self.reconcilees.remove(pubkey);
    }

    pub fn is_reconcilee(&self, pubkey: &PublicKey) -> bool {
        self.reconcilees.contains(pubkey)
    }
}
