use std::sync::Arc;
use std::convert::TryFrom;

use bytes::Bytes;
use failure::Error;

use crate::{crypto::hashes::*, primitives::transaction::*, utils::serialisation::*};

use super::{rocksdb::*, *};

pub trait Storable<U> {
    fn from_db(db: Arc<RocksDb>, id: &Bytes) -> Result<Option<U>, Error>;
    fn to_db(&self, db: Arc<RocksDb>) -> Result<(), Error>;
}

impl Storable<Transaction> for Transaction {
    fn from_db(db: Arc<RocksDb>, tx_id: &Bytes) -> Result<Option<Transaction>, Error> {
        match db.get(tx_id) {
            Ok(Some(some)) => {
                let tx: Transaction = Self::try_from(some)?;
                Ok(Some(tx))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn to_db(&self, db: Arc<RocksDb>) -> Result<(), Error> {
        let tx_id = self.get_id();
        db.put(&tx_id, &Bytes::from(self.clone()))?;
        Ok(())
    }
}
