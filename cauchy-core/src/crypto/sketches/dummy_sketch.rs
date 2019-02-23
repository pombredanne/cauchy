// TODO: Eventually replace this with minisketch and do optimizations

use bytes::Bytes;
use crypto::hashes::blake2b::*;
use std::collections::HashSet;
use std::ops::Sub;
use crypto::sketches::*;

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

impl SketchInsertable for DummySketch {
    fn new() -> DummySketch {
        DummySketch { pos_set: HashSet::new(), neg_set: HashSet::new() }
    }

    fn insert<T: Blk2bHashable>(&mut self, item: &T) {
        self.pos_set.insert(item.blake2b().blake2b());
    }

    fn insert_id(&mut self, item: &Bytes) {
        self.pos_set.insert(item.clone());
    }
}

impl DummySketch {

    pub fn pos_len(&self) -> usize {
        self.pos_set.len()
    }

    pub fn neg_len(&self) -> usize {
        self.pos_set.len()
    }

    pub fn get_pos(&self) -> &HashSet<Bytes> {
        &self.pos_set
    }

    pub fn get_neg(&self) -> &HashSet<Bytes> {
        &self.neg_set
    }
}

impl Decodable for DummySketch {
    fn decode(&self) -> Result<(HashSet<Bytes>, HashSet<Bytes>), String> {
        Ok((self.pos_set.clone(), self.neg_set.clone()))
    }
}