#[macro_use(bson, doc)]
use bson::*;
use std::convert::TryFrom;

use bytes::Bytes;
use failure::Error;

use crate::{crypto::hashes::*, primitives::transaction::*, utils::serialisation::*};

use bson::{bson, doc};
use bson::spec::BinarySubtype;
use super::{mongodb::MongoDB, DataType, Database};

pub trait Storable<U> {
    fn from_db(db: MongoDB, id: &Bytes) -> Result<Option<U>, Error>;
    fn to_db(&self, db: MongoDB) -> Result<(), Error>;
}

impl Storable<Transaction> for Transaction {
    fn from_db(db: MongoDB, tx_id: &Bytes) -> Result<Option<Transaction>, Error> {
        match db.get(&DataType::TX, doc! { "_id" =>  Bson::Binary(BinarySubtype::Generic, tx_id.to_vec())}) {
            Ok(Some(some)) => {
                let tx_data = some.get_binary_generic("v").unwrap();
                let tx: Transaction = Self::try_from(Bytes::from(tx_data.to_vec()))?;
                Ok(Some(tx))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn to_db(&self, db: MongoDB) -> Result<(), Error> {
        let doc = doc! { 
            "_id" => Bson::Binary(BinarySubtype::Generic, self.get_id().to_vec()), 
            "v" => Bson::Binary(BinarySubtype::Generic, Bytes::from(self.clone()).to_vec())
            };
        db.put(&DataType::TX, doc)?;
        Ok(())
    }
}
