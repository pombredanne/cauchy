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
    fn get(
        &self,
        dtype: &DataType,
        doc: bson::ordered::OrderedDocument,
    ) -> Result<Option<bson::ordered::OrderedDocument>, Error> {
        let mut fo = mongodb::coll::options::FindOptions::new();
        fo.sort = Some(doc! { "_id" : -1 });
        match self
            .0
            .collection(dtype.as_str())
            .find_one(Some(doc), Some(fo))
        {
            Ok(Some(found_doc)) => Ok(Some(found_doc)),
            _ => Ok(None),
        }
    }

    // TODO: Handle unhappy path
    fn put(&self, dtype: &DataType, doc: bson::ordered::OrderedDocument) -> Result<(), Error> {
        self.0
            .collection(dtype.as_str())
            .insert_one(doc, None)
            .unwrap();
        Ok(())
    }

    // TODO: Unhappy path
    fn update(
        &self,
        dtype: &DataType,
        filter: bson::ordered::OrderedDocument,
        update: bson::ordered::OrderedDocument,
    ) -> Result<(i32), Error> {
        let n = self
            .0
            .collection(dtype.as_str())
            .update_many(filter, update, None)
            .unwrap()
            .modified_count;
        Ok(n)
    }
}

#[cfg(test)]
impl MongoDB {
    pub fn dropall(&self, dtype: &DataType) {
        self.0.collection(dtype.as_str()).drop().unwrap();
    }
}
