use bytes::Bytes;
use crypto::hashes::blake2b::Blk2bHashable;
use db::rocksdb::RocksDb;
use db::*;
use std::sync::Arc;
use utils::constants::*;
use utils::serialisation::*;

/*
                                      v Auxillary Data               v Binary
[    VarInt    ||    VarInt    ||    Bytes    ||    VarInt    ||    Bytes
       ^UTC            ^ Length of Aux data           ^ Length of Binary
*/

#[derive(Debug, Clone, PartialEq)]
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

    pub fn get_aux(&self) -> &Bytes {
        &self.aux_data
    }

    pub fn get_binary(&self) -> &Bytes {
        &self.binary
    }

    pub fn get_tx_id(&self) -> Bytes {
        Bytes::from(&self.blake2b()[..TX_ID_LEN])
    }

    pub fn get_time(&self) -> &u64 {
        &self.time
    }

    pub fn get_binary_hash(&self) -> Bytes {
        Bytes::from(&self.binary.blake2b()[..TX_ID_LEN])
    }
}
