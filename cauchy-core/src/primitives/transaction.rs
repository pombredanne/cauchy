use bytes::Bytes;
use std::cmp::Ordering;

use crate::crypto::hashes::{blake2b::Blk2bHashable, *};

/*
                                      v Auxillary Data               v Binary
    VarInt    ||    VarInt    ||    Bytes    ||    VarInt    ||    Bytes
       ^UTC            ^ Length of Aux data           ^ Length of Binary
*/

#[derive(Debug, Clone, PartialEq, Eq, Hash)] // TODO: Check if this hash is secure, can this be exploited?
pub struct Transaction {
    time: u64,
    aux_data: Bytes,
    binary: Bytes,
}

impl Transaction {
    pub fn new(time: u64, aux_data: Bytes, binary: Bytes) -> Transaction {
        Transaction {
            time,
            aux_data,
            binary,
        }
    }

    pub fn get_aux(&self) -> Bytes {
        self.aux_data.clone()
    }

    pub fn get_binary(&self) -> Bytes {
        self.binary.clone()
    }

    pub fn get_time(&self) -> u64 {
        self.time
    }

    pub fn get_binary_hash(&self) -> Bytes {
        self.binary.blake2b()
    }
}

impl Identifiable for Transaction {
    fn get_id(&self) -> Bytes {
        self.blake2b().blake2b()
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Transaction) -> Option<Ordering> {
        match self.time.partial_cmp(&other.time) {
            Some(Ordering::Equal) => self.get_id().partial_cmp(&other.get_id()),
            Some(non_equal) => Some(non_equal),
            None => unreachable!()
        }
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Transaction) -> Ordering {
        match self.time.cmp(&other.time) {
            Ordering::Equal => self.get_id().cmp(&other.get_id()),
            other => other
        }    }
}