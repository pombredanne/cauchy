// TODO: Eventually replace this with minisketch and do optimizations

use bytes::Bytes;
use crypto::hashes::blake2b::*;
use std::collections::HashSet;
use std::ops::Sub;

#[derive(Clone, Debug, PartialEq)]
pub struct DummySketch {
    pos_set: HashSet<Bytes>,
    neg_set: HashSet<Bytes>,
}

impl Sub for DummySketch {
    type Output = DummySketch;

    fn sub(self, other: DummySketch) -> DummySketch {
        DummySketch {
            pos_set: self.pos_set.difference(&other.pos_set).cloned().collect(),
            neg_set: other.pos_set.difference(&self.pos_set).cloned().collect(),
        }
    }
}

impl DummySketch {
    pub fn new(pos_set: HashSet<Bytes>, neg_set: HashSet<Bytes>) -> DummySketch {
        DummySketch { pos_set, neg_set }
    }

    pub fn with_capacity(capacity: usize) -> DummySketch {
        DummySketch {
            pos_set: HashSet::with_capacity(capacity),
            neg_set: HashSet::with_capacity(capacity),
        }
    }

    pub fn pos_len(&self) -> usize {
        self.pos_set.len()
    }

    pub fn neg_len(&self) -> usize {
        self.pos_set.len()
    }

    pub fn insert<T: Blk2bHashable>(&mut self, item: &T) {
        self.pos_set.insert(item.blake2b().blake2b());
    }

    pub fn get_pos(&self) -> &HashSet<Bytes> {
        &self.pos_set
    }

    pub fn get_neg(&self) -> &HashSet<Bytes> {
        &self.neg_set
    }

    pub fn decode(&self) -> Result<(HashSet<Bytes>, HashSet<Bytes>), String> {
        Ok((self.pos_set.clone(), self.neg_set.clone()))
    }
}
