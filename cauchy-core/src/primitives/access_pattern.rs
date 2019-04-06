use std::collections::{HashMap, HashSet};
use std::iter::Iterator;
use std::ops::{Add, AddAssign};

use bytes::Bytes;

use crate::utils::byte_ops::*;

#[derive(Clone)]
pub struct AccessPattern {
    pub read: HashSet<Bytes>,
    pub write: HashMap<Bytes, Bytes>
}

impl AddAssign for AccessPattern {
    fn add_assign(&mut self, other: AccessPattern) {
        self.read = self.read.union(&other.read).cloned().collect();
        for (key, value) in other.write {
            match self.write.get(&key) {
                Some(other_value) => self.write.insert(key, value.byte_xor(other_value.clone())),
                None => self.write.insert(key, value),
            };
        }
    }
}

impl AccessPattern {
    pub fn new() -> AccessPattern {
        AccessPattern {
            read: HashSet::new(),
            write: HashMap::new(),
        }
    }

    pub fn commute(&self, other: &AccessPattern) -> bool {
        !(other.read.iter().any(|key| self.write.contains_key(key)) || self.write.keys().any(|key| other.read.contains(key)))
    }
}