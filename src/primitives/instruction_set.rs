use primitives::{instruction::{LocalEvaluation, GlobalEvaluation}, varint::VarInt};
use bytes::Bytes;
use db::Database;
use db::rocksdb::Rocksdb;
use primitives::transaction::Transaction;
use utils::serialisation::SerialisableType;

struct Grab(bool);

impl GlobalEvaluation<Rocksdb> for Grab {
    fn evaluate(&self, db: &Rocksdb, input: &Bytes) -> Result<Bytes, String> {
        if input.len() < 32 {
            return Err("PullMemory arguments too short".to_string())
        }
        let txid = input.slice_to(32);
        let tail = input.slice_from(32);
        let start = usize::from(VarInt::parse(&tail));
        let end = start + usize::from(VarInt::parse(&tail));
        let raw_result = db.get(&txid);
        match raw_result {
            Ok(Some(some)) => {
                let pulled = match Transaction::deserialise(some) {
                    Ok(tx) => tx,
                    Err(error) => return Err(error)
                };
                Transaction::slice(pulled, start, end, self.0)
                },
            Err(error) => Err(error.to_string()),
            Ok(None) => Err("PullMemory couldn't find reference".to_string())
        }
    }
}