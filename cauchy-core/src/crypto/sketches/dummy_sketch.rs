// TODO: Eventually replace this with minisketch
use std::collections::HashSet;
use std::ops::Sub;

use bytes::Bytes;

use super::{super::hashes::*, *};

#[derive(Clone, Debug, PartialEq)]
pub struct DummySketch {
    pos_set: HashSet<Bytes>,
    neg_set: HashSet<Bytes>,
}

impl Sub for DummySketch {
    type Output = DummySketch;

    fn sub(self, other: DummySketch) -> DummySketch {
        let a: HashSet<&Bytes> = self.pos_set.difference(&other.pos_set).collect();
        let b: HashSet<&Bytes> = self.neg_set.difference(&other.neg_set).collect();
        let c: HashSet<&Bytes> = other.pos_set.difference(&self.pos_set).collect();
        let d: HashSet<&Bytes> = other.neg_set.difference(&self.neg_set).collect();
        DummySketch {
            pos_set: a.union(&d).cloned().cloned().collect(),
            neg_set: c.union(&b).cloned().cloned().collect(),
        }
    }
}

impl SketchInsertable for DummySketch {
    fn new() -> DummySketch {
        DummySketch {
            pos_set: HashSet::new(),
            neg_set: HashSet::new(),
        }
    }

    fn insert<T: Identifiable>(&mut self, item: &T) {
        let digest = item.get_id();
        self.pos_set.insert(digest);
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

    pub fn collect(&mut self) {
        self.pos_set = self.pos_set.difference(&self.neg_set).cloned().collect();
        self.neg_set = HashSet::new();
    }
}

impl Decodable for DummySketch {
    fn decode(&self) -> Result<(HashSet<Bytes>, HashSet<Bytes>), String> {
        Ok((self.pos_set.clone(), self.neg_set.clone()))
    }
}

impl From<(HashSet<Bytes>, HashSet<Bytes>)> for DummySketch {
    fn from((pos_set, neg_set): (HashSet<Bytes>, HashSet<Bytes>)) -> DummySketch {
        DummySketch { pos_set, neg_set }
    }
}
