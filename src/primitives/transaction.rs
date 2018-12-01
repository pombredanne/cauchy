use primitives::{varint::VarInt, script::Script};
use bytes::Bytes;

// Structure of Transaction
pub struct Transaction {
    pub n_instructions: VarInt,
    pub instructions: Script,
    pub memory: Bytes,
}

impl Transaction {
    pub fn slice(tx: Transaction, start: usize, end: usize, inst_flag: bool) -> Result<Bytes, String> {
        if inst_flag {
            Ok(Bytes::from(tx.instructions).slice(start, end))
        } else {
            Ok(tx.memory.slice(start, end))
        }
    } 
}