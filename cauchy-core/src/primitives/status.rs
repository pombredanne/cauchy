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
    root: Option<Bytes>,
    ids: Option<HashSet<Bytes>>,
    minisketch: Option<DummySketch>, // Post reconciliation our minisketch should match this
}

impl Expectation {
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
    StatePush,
    StatePull(Expectation),
    WorkPull,
    Fighting(WorkStack),
    Idle,
}

impl Default for Status {
    fn default() -> Self {
        Status::Idle
    }
}

impl Status {
    pub fn to_str(&self) -> &'static str {
        match self {
            Status::StatePush => "state pushing",
            Status::StatePull(_) => "state pulling",
            Status::WorkPull => "work pulling",
            Status::Idle => "idle",
            Status::Fighting(_) => "fighting",
        }
    }
}
