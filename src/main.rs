extern crate blake2;
extern crate bytes;
extern crate rand;
extern crate rocksdb;
extern crate secp256k1;
extern crate tokio;

pub mod consensus;
mod crypto;
pub mod daemon;
pub mod db;
pub mod net;
pub mod primitives;
pub mod utils;

use rand::Rng;
use bytes::Bytes;
use consensus::status::Status;
use crypto::hashes::odd_sketch::*;
use crypto::signatures::ecdsa;
use db::rocksdb::RocksDb;
use db::*;
use primitives::work_site::*;
use std::sync::Arc;
use std::thread;
use utils::constants::TX_DB_PATH;
use utils::mining;
use std::time;

#[cfg(test)]
mod test {
    mod byte_op_tests;
    mod db_tests;
    mod hash_tests;
    mod signature_tests;
    mod transaction_state_tests;
    mod transaction_tests;
    mod varint_tests;
}

fn main() {

    let tx_db = Arc::new(RocksDb::open_db(TX_DB_PATH).unwrap());

    let mut state = vec![
        Bytes::from(&b"a"[..]),
        Bytes::from(&b"b"[..]),
        Bytes::from(&b"c"[..]),
        Bytes::from(&b"d"[..]),
        Bytes::from(&b"e"[..]),
        Bytes::from(&b"f"[..]),
    ];

    let mut state_sketch = state.odd_sketch();
    let (sk, pk) = ecdsa::generate_keypair();
    let my_status = Arc::new(Status::new(state_sketch.clone(), &WorkSite::init(pk)));

    let mining_work_site = Arc::new(WorkSite::init(pk));

    let my_status_arc = Arc::clone(&my_status);
    let my_work_site_arc = Arc::clone(&mining_work_site);

    thread::spawn(move || mining::mine(my_work_site_arc.clone(), my_status_arc.clone()));

    let my_status_arc = Arc::clone(&my_status);
    let my_work_site_arc = Arc::clone(&mining_work_site);


    thread::spawn(move || daemon::response_server(tx_db.clone(), my_status_arc, Arc::new(sk)));

    let ten_millis = time::Duration::from_millis(10);

    loop {
        thread::sleep(ten_millis); // TODO: Remove
        state.push(random_tx());
        state_sketch = state.odd_sketch();
        my_status.update_state_sketch(state_sketch);
    }

    fn random_tx() -> Bytes {
        let mut rng = rand::thread_rng();
        let my_array: [u8; 8] = rng.gen();
        Bytes::from(&my_array[..])
    }


}
