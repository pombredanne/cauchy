use std::convert::TryFrom;
use std::sync::Arc;

use bytes::Bytes;
use failure::Error;

use crate::{crypto::hashes::*, primitives::transaction::*, utils::serialisation::*};

use super::{mongodb::*, *};

pub trait Storable<U> {
    fn from_db(db: MongoDB, id: &Bytes) -> Result<Option<U>, Error>;
    fn to_db(&self, db: MongoDB) -> Result<(), Error>;
}

impl Storable<Transaction> for Transaction {
    fn from_db(db: MongoDB, tx_id: &Bytes) -> Result<Option<Transaction>, Error> {
        match db.get(&DataType::TX, tx_id) {
            Ok(Some(some)) => {
                let tx: Transaction = Self::try_from(some)?;
                Ok(Some(tx))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn to_db(&self, db: MongoDB) -> Result<(), Error> {
        let tx_id = self.get_id();
        db.put(&DataType::TX, &tx_id, &Bytes::from(self.clone()))?;
        Ok(())
    }
}
