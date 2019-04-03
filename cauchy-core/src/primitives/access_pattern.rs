use std::collections::{HashMap, HashSet};
use std::ops::Add;
use std::iter::Iterator;

use bytes::Bytes;

use crate::utils::byte_ops::*;

pub struct ReadPattern(HashSet<Bytes>);

impl Add for ReadPattern {
    type Output = ReadPattern;

    fn add(self, other: ReadPattern) -> ReadPattern {
        ReadPattern(self.0.union(&other.0).cloned().collect())
    }
}

impl ReadPattern {
    pub fn disjoint(&self, other: &ReadPattern) -> bool {
        self.0.is_disjoint(&other.0)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<Bytes> {
        self.0.iter()
    }

    pub fn empty() -> ReadPattern {
        ReadPattern(HashSet::with_capacity(0))
    }
}

pub struct Delta(HashMap<Bytes, Bytes>);

impl Add for Delta {
    type Output = Delta;

    fn add(self, other: Delta) -> Delta {
        let mut new_map = self.0;
        for (key, value) in other.0 {
            match new_map.get(&key) {
                Some(other_value) => new_map.insert(key, value.byte_xor(other_value.clone())),
                None => new_map.insert(key, value),
            };
        }
        Delta(new_map)
    }
}

impl Delta {
    pub fn disjoint(&self, other: &Delta) -> bool {
        !other.0.keys().any(|key| self.0.contains_key(key))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<Bytes, Bytes> {
        self.0.iter()
    }

    pub fn empty() -> Delta {
        Delta(HashMap::with_capacity(0))
    }
}

pub struct AccessPattern {
    pub read_pattern: ReadPattern,
    pub delta: Delta
}

impl Add for AccessPattern {
    type Output = AccessPattern;

    fn add(self, other: AccessPattern) -> AccessPattern {
        AccessPattern {
            read_pattern: self.read_pattern + other.read_pattern,
            delta: self.delta + other.delta
        }
    }
}

impl AccessPattern {
    pub fn empty() -> AccessPattern {
        AccessPattern {
            read_pattern: ReadPattern::empty(),
            delta: Delta::empty()
        }
    }
}