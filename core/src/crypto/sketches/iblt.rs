// TODO: Eventually replace this with minisketch and do optimizations

use bytes::Bytes;
use crypto::hashes::blake2b::*;
use crypto::sketches::odd_sketch::*;
use crypto::util::*;
use std::collections::HashSet;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use utils::byte_ops::*;
use utils::constants::*;

#[derive(PartialEq, Clone, Debug)]
pub struct Row {
    count: i32,
    payload: Bytes,
    checksum: Bytes,
}

impl Row {
    pub fn empty_row() -> Row {
        Row {
            count: 0,
            payload: Bytes::from(&[0; IBLT_PAYLOAD_LEN][..]),
            checksum: Bytes::from(&[0; IBLT_CHECKSUM_LEN][..]),
        }
    }

    pub fn new(count: i32, payload: Bytes, checksum: Bytes) -> Row {
        Row {
            count,
            payload: payload.slice_to(IBLT_PAYLOAD_LEN),
            checksum: checksum.slice_to(IBLT_CHECKSUM_LEN),
        }
    }

    pub fn get_count(&self) -> i32 {
        self.count
    }

    pub fn get_payload(&self) -> Bytes {
        self.payload.clone()
    }

    pub fn get_checksum(&self) -> Bytes {
        self.checksum.clone()
    }

    pub fn unit_row(payload: &Bytes) -> Row {
        Row {
            count: 1,
            payload: payload.clone().slice_to(IBLT_PAYLOAD_LEN),
            checksum: payload.blake2b().slice_to(IBLT_CHECKSUM_LEN),
        }
    }

    pub fn count_row(payload: &Bytes, count: i32) -> Row {
        Row {
            count,
            payload: payload.clone().slice_to(IBLT_PAYLOAD_LEN),
            checksum: payload.blake2b().slice_to(IBLT_CHECKSUM_LEN),
        }
    }

    pub fn is_pure(&self) -> bool {
        (self.count == 1 || self.count == -1)
            && (self.checksum == self.payload.blake2b().slice_to(IBLT_CHECKSUM_LEN))
    }

    pub fn is_empty(&self) -> bool {
        (self.count == 0)
            && (self.payload.iter().all(|&x| x == 0))
            && (self.checksum.iter().all(|&x| x == 0))
    }
}

impl Add for Row {
    type Output = Row;

    fn add(self, other: Row) -> Row {
        Row {
            count: self.count + other.count,
            payload: Bytes::byte_xor(self.payload, other.payload),
            checksum: Bytes::byte_xor(self.checksum, other.checksum),
        }
    }
}

impl AddAssign for Row {
    fn add_assign(&mut self, other: Row) {
        *self = Row {
            count: self.count + other.count,
            payload: Bytes::byte_xor(self.payload.clone(), other.payload),
            checksum: Bytes::byte_xor(self.checksum.clone(), other.checksum),
        };
    }
}

impl Sub for Row {
    type Output = Row;

    fn sub(self, other: Row) -> Row {
        Row {
            count: self.count - other.count,
            payload: Bytes::byte_xor(self.payload, other.payload),
            checksum: Bytes::byte_xor(self.checksum, other.checksum),
        }
    }
}

impl SubAssign for Row {
    fn sub_assign(&mut self, other: Row) {
        *self = Row {
            count: self.count - other.count,
            payload: Bytes::byte_xor(self.payload.clone(), other.payload),
            checksum: Bytes::byte_xor(self.checksum.clone(), other.checksum),
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IBLT {
    n_hashes: usize,
    rows: Vec<Row>,
}

impl Sub for IBLT {
    type Output = IBLT;

    fn sub(self, other: IBLT) -> IBLT {
        IBLT {
            n_hashes: self.n_hashes,
            rows: self
                .rows
                .into_iter()
                .zip(other.rows.into_iter())
                .map(|(row_a, row_b)| row_a - row_b)
                .collect(),
        }
    }
}

impl IBLT {
    pub fn with_capacity(capacity: usize, n_hashes: usize) -> IBLT {
        IBLT {
            n_hashes,
            rows: vec![Row::empty_row(); capacity],
        }
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn from_rows(rows: Vec<Row>, n_hashes: usize) -> IBLT {
        IBLT { n_hashes, rows }
    }

    pub fn get_rows(&self) -> &Vec<Row> {
        &self.rows
    }

    pub fn is_empty(&self) -> bool {
        self.rows.iter().all(|row| row.is_empty())
    }

    pub fn get_pure(&self) -> Option<Row> {
        self.rows.iter().find(|row| row.is_pure()).cloned()
    }

    pub fn insert(&mut self, payload: Bytes) {
        let len = self.rows.len();
        for i in (0..self.n_hashes).map(|k| get_pos(&payload, k, len)) {
            self.rows[i] = self.rows[i].clone() + Row::unit_row(&payload);
        }
    }

    pub fn decode(&self) -> Result<(HashSet<Bytes>, HashSet<Bytes>), String> {
        let mut left = HashSet::with_capacity(self.rows.len());
        let mut right = HashSet::with_capacity(self.rows.len());

        let mut decode_iblt = self.clone();

        loop {
            if let Some(row) = decode_iblt.get_pure() {
                let payload = row.clone().payload;
                let count = row.clone().count;

                if count > 0 {
                    left.insert(payload.clone());
                } else {
                    right.insert(payload.clone());
                }

                for j in (0..self.n_hashes).map(|k| get_pos(&payload, k, self.rows.len())) {
                    decode_iblt.rows[j] -= Row::count_row(&payload, count);
                }
            } else {
                break;
            }
        }

        if decode_iblt.is_empty() {
            Ok((left, right))
        } else {
            Err("Failed to decode IBLT".to_string())
        }
    }
}

impl PartialEq<Bytes> for IBLT {
    fn eq(&self, other: &Bytes) -> bool {
        let (hash_set_l, _) = match self.decode() {
            Ok(hs) => hs,
            Err(_) => return false,
        };
        *other == hash_set_l.odd_sketch()
    }
}
