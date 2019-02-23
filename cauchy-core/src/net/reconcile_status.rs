use bytes::Bytes;
use crypto::signatures::ecdsa::generate_dummy_pubkey;
use primitives::transaction::*;
use secp256k1::PublicKey;
use std::collections::HashSet;
use crypto::hashes::*;

pub struct ReconciliationStatus {
    live: bool,
    target: PublicKey,
    payload_ids: HashSet<Bytes>,
    reconcilees: HashSet<PublicKey>
}

impl ReconciliationStatus {
    pub fn new() -> ReconciliationStatus {
        let dummy_pk = generate_dummy_pubkey();
        ReconciliationStatus {
            live: false,
            target: dummy_pk,
            payload_ids: HashSet::new(),
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

    pub fn ids_eq(&self, other: &HashSet<Transaction>) -> bool {
        let received_tx_ids: HashSet<Bytes> = other.iter().map(move |tx| tx.get_id()).collect();
        self.payload_ids == received_tx_ids
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
