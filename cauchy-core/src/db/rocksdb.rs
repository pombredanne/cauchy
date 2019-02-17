use bytes::Bytes;
use db::Database;
use failure::Error;
use rocksdb::DB;
use utils::errors::DatabaseError;

pub struct RocksDb(DB);

impl Database<RocksDb> for RocksDb {
    fn open_db(folder: &str) -> Result<RocksDb, Error> {
        let mut path = match dirs::home_dir() {
            Some(some) => some,
            None => return Err(DatabaseError::DbPath.into()),
        };
        path.push(folder);
        match DB::open_default(path) {
            Ok(some) => Ok(RocksDb(some)),
            Err(err) => Err(DatabaseError::Open.into()),
        }
    }

    fn get(&self, key: &Bytes) -> Result<Option<Bytes>, Error> {
        match self.0.get(key) {
            Ok(Some(some)) => Ok(Some(Bytes::from(&*some))),
            Ok(None) => Ok(None),
            Err(err) => Err(DatabaseError::Open.into()),
        }
    }

    fn put(&self, key: &Bytes, value: &Bytes) -> Result<(), Error> {
        match self.0.put(key, value) {
            Ok(_) => Ok(()),
            Err(err) => Err(DatabaseError::Put.into()),
        }
    }
}
