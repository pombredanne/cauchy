extern crate dirs;
use bytes::Bytes;

//pub mod memcache;
pub mod rocksdb;

pub const TX_DB_PATH: &str = ".geodesic/db/";
pub const STATE_DB_PATH: &str = ".geodesic/db/";

pub trait Database<DB> {
    fn open_db(path: &str) -> Result<DB, String>;
    fn get(&self, key: &Bytes) -> Result<Option<Bytes>, String>;
    fn put(&self, key: &Bytes, value: &Bytes) -> Result<(), String>;
}
