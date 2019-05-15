extern crate dirs;

pub mod mongodb;
pub mod storing;

use failure::Error;

pub enum DataType {
    TX,
    State,
}

pub trait Database<DB> {
    fn open_db(path: &str) -> Result<DB, Error>;
    fn put(&self, dtype: &DataType, doc: bson::ordered::OrderedDocument) -> Result<(), Error>;
    fn get(
        &self,
        dtype: &DataType,
        doc: bson::ordered::OrderedDocument,
    ) -> Result<Option<bson::ordered::OrderedDocument>, Error>;
    fn update(
        &self,
        dtype: &DataType,
        filter: bson::ordered::OrderedDocument,
        update: bson::ordered::OrderedDocument,
    ) -> Result<(i32), Error>;
}

impl DataType {
    pub fn as_str(&self) -> &str {
        match self {
            &DataType::TX => "txs",
            &DataType::State => "states",
        }
    }
}
