use bytes::Bytes;
use crypto::hashes::blake2b::Blk2bHashable;
use primitives::script::Script;
use utils::constants::*;

/*
                  v Length of next script
VarInt || VarInt || VarInt || Script || VarInt || Script || ... || VarInt || Script
  ^UTC      ^ Number of spendable scripts                            ^ Length of next script

-First script is executed and must return true, the others are added to the "library".
-The scripts are segmented into spendable and referencable,
    the divider is given by the Number of spendable script field.
-Number of scripts should be bounded at 256 (as VarInt size is bounded at 64 bytes).
*/

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    time: u64,
    n_spendable: u32,
    scripts: Vec<Script>,
}

impl Transaction {
    pub fn new(time: u64, n_spendable: u32, scripts: Vec<Script>) -> Self {
        Transaction {
            time,
            n_spendable,
            scripts,
        }
    }

    pub fn get_script(&self, i: usize) -> Result<&Script, String> {
        if self.scripts.len() < i {
            Err("Script out of range".to_string())
        } else {
            Ok(&self.scripts[i])
        }
    }

    pub fn tx_id(&self) -> Bytes {
        Bytes::from(&self.blake2b()[..TX_ID_LEN])
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn n_spendable(&self) -> u32 {
        self.n_spendable
    }
}

impl From<Transaction> for Vec<Script> {
    fn from(tx: Transaction) -> Vec<Script> {
        tx.scripts
    }
}
