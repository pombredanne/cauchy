use bytes::Bytes;
use db::rocksdb::*;
use db::*;
use primitives::transaction::*;
use std::sync::Arc;
use utils::serialisation::*;

pub trait Storable<U> {
    fn from_db(db: Arc<RocksDb>, id: &Bytes) -> Result<Option<U>, String>;
    fn to_db(&self, db: Arc<RocksDb>) -> Result<(), String>;
}

impl Storable<Transaction> for Transaction {
    fn from_db(db: Arc<RocksDb>, tx_id: &Bytes) -> Result<Option<Transaction>, String> {
        match db.get(tx_id) {
            Ok(Some(some)) => {
                let tx: Transaction = Self::try_from(some)?;
                Ok(Some(tx))
            }
            Ok(None) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    fn to_db(&self, db: Arc<RocksDb>) -> Result<(), String> {
        let tx_id = self.get_tx_id();
        db.put(&tx_id, &Bytes::from(self.clone()))?;
        Ok(())
    }
}
