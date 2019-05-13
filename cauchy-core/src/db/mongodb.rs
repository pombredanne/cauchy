#[macro_use(bson, doc)]
use bson::*;
use bytes::Bytes;
use failure::Error;
use mongodb::db::*;
use mongodb::{bson, doc, Client, ThreadedClient};
use std::sync::Arc;

use super::{DataType, Database};
use crate::utils::errors::SystemError;

#[derive(Clone)]
pub struct MongoDB(Arc<mongodb::db::DatabaseInner>);

impl Database<MongoDB> for MongoDB {
    fn open_db(name: &str) -> Result<MongoDB, Error> {
        let db = match Client::connect("localhost", 27017) {
            Ok(c) => MongoDB(c.db(name)),
            Err(_) => return Err(SystemError::InvalidPath.into()),
        };
        Ok(db)
    }

    // TODO: Handle unhappy path
    fn get(&self, dtype: &DataType, key: &Bytes) -> Result<Option<Bytes>, Error> {
        let doc = doc! { "_id" =>  Bson::Binary(bson::spec::BinarySubtype::Generic, key.to_vec())};
        match self.0.collection(dtype.as_str()).find_one(Some(doc), None) {
            Ok(Some(found_doc)) => {
                let val_binary = found_doc.get_binary_generic("val").unwrap();
                let bytes = Bytes::from(val_binary.to_vec());
                Ok(Some(bytes))
            }
            _ => Ok(None),
        }
    }

    // TODO: Handle unhappy path
    fn put(&self, dtype: &DataType, key: &Bytes, value: &Bytes) -> Result<(), Error> {
        self.0
            .collection(dtype.as_str())
            .insert_one(
                doc! { "_id" => Bson::Binary(bson::spec::BinarySubtype::Generic, key.to_vec()), "val" => Bson::Binary(bson::spec::BinarySubtype::Generic, value.to_vec()) },
                None,
            )
            .unwrap();
        Ok(())
    }
}
