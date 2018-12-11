use bytes::Bytes;
use db::Database;
use rocksdb::DB;

pub struct Rocksdb(DB);

impl Database<Rocksdb> for Rocksdb {
    fn open_db(folder: &str) -> Result<Rocksdb, String> {
        let mut path = match dirs::home_dir() {
            Some(some) => some,
            None => return Err("No home directory found".to_string()),
        };
        path.push(folder);
        match DB::open_default(path) {
            Ok(some) => Ok(Rocksdb(some)),
            Err(error) => Err(error.to_string()),
        }
    }

    fn get(&self, key: &Bytes) -> Result<Option<Bytes>, String> {
        match self.0.get(key) {
            Ok(Some(some)) => Ok(Some(Bytes::from(&*some))),
            Ok(None) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    fn put(&self, key: &Bytes, value: &Bytes) -> Result<(), String> {
        match self.0.put(key, value) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}
