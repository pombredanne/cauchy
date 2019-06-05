use std::convert::TryFrom;

use bytes::Bytes;
use failure::Error;

use crate::{crypto::hashes::*, primitives::transaction::*, vm::session::Session};

use super::{mongodb::MongoDB, DataType, Database};
use bson::spec::BinarySubtype;
use bson::{bson, doc, Bson};

pub trait Storable
where
    Self: Sized,
{
    type Context;
    fn from_db(context: &mut Self::Context, id: Bytes) -> Result<Option<Self>, Error>;
    fn to_db(&self, context: &mut Self::Context, key: Option<Bytes>) -> Result<(), Error>;
}

impl Storable for Transaction {
    type Context = MongoDB;
    fn from_db(db: &mut MongoDB, tx_id: Bytes) -> Result<Option<Transaction>, Error> {
        match db.get(
            &DataType::TX,
            doc! { "_id" =>  Bson::Binary(BinarySubtype::Generic, tx_id.to_vec())},
        ) {
            Ok(Some(some)) => {
                let tx_data = some.get_binary_generic("v").unwrap();
                let tx: Transaction = Self::try_from(Bytes::from(tx_data.to_vec()))?;
                Ok(Some(tx))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn to_db(&self, db: &mut MongoDB, _key: Option<Bytes>) -> Result<(), Error> {
        let doc = doc! {
        "_id" => Bson::Binary(BinarySubtype::Generic, self.get_id().to_vec()),
        "v" => Bson::Binary(BinarySubtype::Generic, Bytes::from(self.clone()).to_vec())
        };
        db.put(&DataType::TX, doc)?;
        Ok(())
    }
}

pub struct ValueStore(pub Bytes);

impl Storable for ValueStore {
    type Context = Session;
    fn from_db(context: &mut Session, key: Bytes) -> Result<Option<ValueStore>, Error> {
        let doc = doc! {
            "t" : Bson::Binary(BinarySubtype::Generic, context.id.to_vec()),
            "$or" : [
                { "p" :  Bson::Binary(BinarySubtype::Generic, context.perfid.to_vec()) },
                { "p" :  Bson::Null },
                { "p" : {"$exists" : false}},
            ],
            "k" : Bson::Binary(BinarySubtype::Generic, key.to_vec()),
        };
        println!("{:?}", doc);
        context
            .performance
            .lock()
            .unwrap()
            .add_read(&context.id, key);
        match context.store.get(&DataType::State, doc) {
            Ok(Some(some)) => {
                println!("res: {:?}", some);
                Ok(Some(ValueStore(Bytes::from(
                    &some.get_binary_generic("v").unwrap()[..],
                ))))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn to_db(&self, context: &mut Session, key: Option<Bytes>) -> Result<(), Error> {
        let key = match key {
            Some(some) => some,
            None => unreachable!(), // TODO: Throw appropriate error
        };
        let mut store_id = context.timestamp.to_be_bytes().to_vec();
        store_id.append(&mut context.id.to_vec());
        store_id.append(&mut key.to_vec());
        let doc = doc! {
            // "_id" => Bson::Binary(BinarySubtype::Generic, store_id),
            // The [t]xid this item belongs to
            "t" => Bson::Binary(BinarySubtype::Generic, context.id.to_vec()),
            // The [o]riginating txid
            "o" => Bson::Binary(BinarySubtype::Generic, context.perfid.to_vec()),
            // The current [p]erformance id (unset once the performance is accepted)
            "p" => Bson::Binary(BinarySubtype::Generic, context.perfid.to_vec()),
            // The [k]ey for this value, as provided by the script
            "k" => Bson::Binary(BinarySubtype::Generic, key.to_vec()),
            // The [v]alue associated with this key, as provided by the script
            "v" => Bson::Binary(BinarySubtype::Generic, self.0.to_vec()),
        };
        context
            .performance
            .lock()
            .unwrap()
            .add_write(&context.id, key, self.0.clone());
        context.store.put(&DataType::State, doc)
        // Ok(())
    }
}
