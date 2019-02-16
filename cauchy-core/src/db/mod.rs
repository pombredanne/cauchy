extern crate dirs;
use bytes::Bytes;

//pub mod memcache;
pub mod rocksdb;
pub mod storing;

use failure::Error;

pub trait Database<DB> {
    fn open_db(path: &str) -> Result<DB, Error>;
    fn get(&self, key: &Bytes) -> Result<Option<Bytes>, Error>;
    fn put(&self, key: &Bytes, value: &Bytes) -> Result<(), Error>;
}
