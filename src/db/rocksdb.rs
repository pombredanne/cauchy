use db::Database;
use rocksdb::{DB};
use bytes::Bytes;
extern crate dirs;

pub struct Rocksdb{
    db: DB,
}

const ROCKS_DB_PATH : &str = "/.saturn/db/";

impl Database<Rocksdb> for Rocksdb {
    fn open_db() -> Result<Rocksdb, String> {
        let result = DB::open_default(ROCKS_DB_PATH);
        match result {
            Ok(some) => {
                Ok(Rocksdb{db: some})
            },
            Err(error) => Err(error.to_string()),
        }
    }

    fn get(&self, key: &Bytes) -> Result<Option<Bytes>, String> {
        match self.db.get(key) {
            Ok(Some(some)) => Ok(Some(Bytes::from(&*some))),
            Ok(None) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    fn put(&self, key: &Bytes, value: &Bytes) -> Result<(), String> {
        match self.db.put(key, value) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.to_string())
        }
    }
}