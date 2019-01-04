extern crate blake2;
extern crate bus;
extern crate bytes;
extern crate crossbeam;
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
pub mod state;
pub mod utils;

use bus::Bus;
use bytes::Bytes;
use crypto::signatures::ecdsa;
use crypto::sketches::odd_sketch::*;
use primitives::script::Script;
use primitives::status::Status;
use primitives::transaction::Transaction;
use utils::mining;

use db::rocksdb::RocksDb;
use db::*;
use utils::constants::TX_DB_PATH;

use crossbeam::channel;
use rand::Rng;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time;

#[cfg(test)]
mod test {
    mod byte_op_tests;
    mod db_tests;
    mod hash_tests;
    mod signature_tests;
    mod sketch_tests;
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

    let (sk, pk) = ecdsa::generate_keypair();

    let (distance_send, distance_recv) = channel::unbounded();
    let mut odd_sketch_bus = Bus::new(10);
    let n_mining_threads: u64 = 1;
    
    for i in 0..n_mining_threads {
        let distance_send_c = distance_send.clone();
        let mut sketch_recv = odd_sketch_bus.add_rx();

        thread::spawn(move || {
            mining::mine(
                pk,
                sketch_recv,
                distance_send_c,
                i,
                n_mining_threads
            )
        });
    }

    let status = Arc::new(Status::null());

    // Server
    let status_c = status.clone();
    thread::spawn(move || daemon::server(tx_db, status_c, pk, sk));

    // Update local state
    let (sketch_send, sketch_recv) = channel::unbounded();
    thread::spawn(move || status.update_local(odd_sketch_bus, sketch_recv, distance_recv));

    let new_tx_interval = time::Duration::from_millis(100);

    loop {
        sketch_send.send(random_tx());
        thread::sleep(new_tx_interval);
    }

    fn random_tx() -> Transaction {
        let mut rng = rand::thread_rng();
        let my_array: [u8; 8] = rng.gen();
        let raw_script = Bytes::from(&my_array[..]);
        Transaction::new(0, 0, vec![Script::new(raw_script)])
    }
}
