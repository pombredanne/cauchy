use db::Database;
use memcache::{Client, MemcacheError};
use bytes::Bytes;

struct Memcache {
    db: Client,
}

const MEMCACHE_IP : &str = "memcache://127.0.0.1:11211?timeout=10&tcp_nodelay=true";

// impl Database<Memcache, MemcacheError> for Memcache {
//     fn open_db() -> Result<Memcache, MemcacheError> {
//         let result = Client::new(MEMCACHE_IP);
//         match result {
//             Ok(some) => {
//                 Ok(Memcache{db: some})
//             },
//             Err(error) => Err(error),
//         }
//     }

//     fn get(&self, key: Bytes) -> Result<Option<Bytes>, Error> {
//         match self.db.get(&key.) {
//             Ok(Some(some)) => Ok(Some(Bytes::from(&*some))),
//             Ok(None) => Ok(None),
//             Err(error) => Err(error),
//         }
//     }

//     fn put(&self, key: Bytes, value: Bytes) -> Result<(), Error> {
//         self.db.put(&key, &value)
//     }
// }