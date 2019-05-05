extern crate dirs;
use bytes::Bytes;

pub mod mongodb;
pub mod storing;

use failure::Error;

pub enum DataType {
    TX,
    State
}

pub trait Database<DB> {
    fn open_db(path: &str) -> Result<DB, Error>;
    fn get(&self, dtype: &DataType, key: &Bytes) -> Result<Option<Bytes>, Error>;
    fn put(&self, dtype: &DataType, key: &Bytes, value: &Bytes) -> Result<(), Error>;
}

impl DataType {
    pub fn as_str(&self) -> &str {
        match self {
            &DataType::TX => "txs",
            &DataType::State => "states",
        }
    }
}