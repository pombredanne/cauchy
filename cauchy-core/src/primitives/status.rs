use std::collections::HashSet;

use crate::{
    crypto::{
        hashes::Identifiable,
        sketches::{dummy_sketch::DummySketch, odd_sketch::OddSketch},
    },
    primitives::transaction::Transaction,
};

use super::work::WorkStack;

use bytes::Bytes;

#[derive(Default, PartialEq, Clone)]
pub struct Expectation {
    oddsketch: OddSketch,
    root: Bytes,
    ids: Option<HashSet<Bytes>>,
    minisketch: Option<DummySketch>, // Post reconciliation our minisketch should match this
}

impl Expectation {
    pub fn new(oddsketch: OddSketch, root: Bytes) -> Expectation {
        Expectation {
            oddsketch,
            root,
            ids: None,
            minisketch: None
        }
    }

    pub fn get_ids(&self) -> Option<HashSet<Bytes>> {
        self.ids.clone()
    }

    pub fn get_oddsketch(&self) -> OddSketch {
        self.oddsketch.clone()
    }

    pub fn get_minisketch(&self) -> Option<DummySketch> {
        self.minisketch.clone()
    }

    pub fn update_ids(&mut self, ids: HashSet<Bytes>) {
        self.ids = Some(ids)
    }

    pub fn update_minisketch(&mut self, minisketch: DummySketch) {
        self.minisketch = Some(minisketch)
    }

    pub fn is_expected_payload(&self, transactions: &HashSet<Transaction>) -> bool {
        Some(transactions.iter().map(|tx| tx.get_id()).collect()) == self.ids
    }

    pub fn clear_ids(&mut self) {
        self.ids = None
    }

    pub fn clear_minisketch(&mut self) {
        self.minisketch = None
    }
}

#[derive(PartialEq, Clone)]
pub enum Status {
    Pulling,
    Idle
}

impl Default for Status {
    fn default() -> Self {
        Status::Idle
    }
}


#[derive(PartialEq, Clone)]
pub enum PeerStatus {
    StatePush,
    StatePull(Expectation),
    WorkPull,
    Fighting(WorkStack),
    Idle,
}

impl Default for PeerStatus {
    fn default() -> Self {
        PeerStatus::Idle
    }
}

impl PeerStatus {
    pub fn to_str(&self) -> &'static str {
        match self {
            PeerStatus::StatePush => "state pushing",
            PeerStatus::StatePull(_) => "state pulling",
            PeerStatus::WorkPull => "work pulling",
            PeerStatus::Idle => "idle",
            PeerStatus::Fighting(_) => "fighting",
        }
    }
}
