use bytes::Bytes;
use db::Database;
use failure::Error;
use rocksdb::DB;
use utils::errors::SystemError;

pub struct RocksDb(DB);

impl Database<RocksDb> for RocksDb {
    fn open_db(folder: &str) -> Result<RocksDb, Error> {
        let mut path = match dirs::home_dir() {
            Some(some) => some,
            None => return Err(SystemError::InvalidPath.into()),
        };
        path.push(folder);
        Ok(RocksDb(DB::open_default(path)?))
    }

    fn get(&self, key: &Bytes) -> Result<Option<Bytes>, Error> {
        match self.0.get(key)? {
            Some(some) => Ok(Some(Bytes::from(&*some))),
            None => Ok(None),
        }
    }

    fn put(&self, key: &Bytes, value: &Bytes) -> Result<(), Error> {
        Ok(self.0.put(key, value)?)
    }
}
