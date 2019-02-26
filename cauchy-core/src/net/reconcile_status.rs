use bytes::Bytes;
use crypto::signatures::ecdsa::generate_dummy_pubkey;
use primitives::transaction::*;
use secp256k1::PublicKey;
use std::collections::HashSet;
use crypto::hashes::*;
use std::sync::Arc;
use primitives::status::*;

use crypto::sketches::*;
use crypto::sketches::odd_sketch::*;
use crypto::sketches::dummy_sketch::*;

pub struct ReconciliationStatus {
    live: bool,
    target: PublicKey,
    missing_ids: HashSet<Bytes>,
    excess_ids: HashSet<Bytes>,
    reconcilees: HashSet<PublicKey>
}

impl ReconciliationStatus {
    pub fn new() -> ReconciliationStatus {
        let dummy_pk = generate_dummy_pubkey();
        ReconciliationStatus {
            live: false,
            target: dummy_pk,
            missing_ids: HashSet::new(),
            excess_ids: HashSet::new(),
            reconcilees: HashSet::new()
        }
    }

    pub fn is_live(&self) -> bool {
        self.live
    }

    pub fn stop(&mut self) {
        self.live = false;
    }

    pub fn set_target(&mut self, new_target: &PublicKey) {
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
        let perceived_odd_sketch = perception.get_odd_sketch();
        let perceived_mini_sketch = perception.get_mini_sketch();
        local_status.update_odd_sketch(
            perceived_odd_sketch
            .xor(&OddSketch::sketch_ids(&self.excess_ids))
            .xor(&OddSketch::sketch_ids(&self.missing_ids))
        );
        
        let mut new_mini_sketch = perceived_mini_sketch - DummySketch::from((self.excess_ids.clone(), self.missing_ids.clone()));
        new_mini_sketch.collect();
        local_status.update_mini_sketch(new_mini_sketch)

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
