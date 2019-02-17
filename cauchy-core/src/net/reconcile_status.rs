use crypto::signatures::ecdsa::generate_dummy_pubkey;
use secp256k1::PublicKey;
use std::collections::HashSet;
use bytes::Bytes;
use primitives::transaction::*;
use crypto::hashes::blake2b::*;

pub struct ReconciliationStatus {
    live: bool,
    target: PublicKey,
    payload_ids: HashSet<Bytes>
}

impl ReconciliationStatus {
    pub fn new() -> ReconciliationStatus {
        let dummy_pk = generate_dummy_pubkey();
        ReconciliationStatus {
            live: false,
            target: dummy_pk,
            payload_ids: HashSet::new(),
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
        self.target == *other
    }

    pub fn ids_eq(&self, other: &HashSet<Transaction>) -> bool {
        let received_tx_ids: HashSet<Bytes> = other.iter().map(move |tx| tx.get_id()).collect();
        self.payload_ids == received_tx_ids
    }
}