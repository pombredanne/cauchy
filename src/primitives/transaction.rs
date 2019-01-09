use bytes::Buf;
use bytes::Bytes;
use crypto::hashes::blake2b::Blk2bHashable;
use primitives::script::Script;
use primitives::varint::VarInt;
use utils::constants::*;

/*
                       v Number of referancable scripts
VarInt || VarInt || VarInt || VarInt || Script || VarInt || Script || ... || VarInt || Script
  ^UTC      ^ Number of spendable scripts            ^ Length of next script

-First script is executed and must return true, the others are added to the "library".
-The scripts are segmented into spendable and referencable,
    the divider is given by the Number of spendable script field.
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

    pub fn parse_buf<T: Buf>(buf: &mut T, len: usize) -> Result<Transaction, String> {
        let mut scripts = Vec::new();

        let vi_time = VarInt::parse_buf(buf)?;
        let n_spendable = VarInt::parse_buf(buf)?;

        for _ in 0..len {
            let vi = VarInt::parse_buf(buf)?;

            let len = usize::from(vi);
            let mut dst = vec![0; len as usize];
            buf.copy_to_slice(&mut dst);

            scripts.push(Script::new(Bytes::from(dst)));
        }
        Ok(Transaction::new(
            u64::from(vi_time),
            u32::from(n_spendable),
            scripts,
        ))
    }
}

impl From<Transaction> for Vec<Script> {
    fn from(tx: Transaction) -> Vec<Script> {
        tx.scripts
    }
}
