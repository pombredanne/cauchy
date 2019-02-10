extern crate dirs;
use bytes::Bytes;

//pub mod memcache;
pub mod rocksdb;
pub mod storing;

pub trait Database<DB> {
    fn open_db(path: &str) -> Result<DB, String>;
    fn get(&self, key: &Bytes) -> Result<Option<Bytes>, String>;
    fn put(&self, key: &Bytes, value: &Bytes) -> Result<(), String>;
}
