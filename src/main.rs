extern crate blake2;
extern crate bus;
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
pub mod state;
pub mod utils;

use bus::Bus;
use bytes::Bytes;
use crypto::signatures::ecdsa;
use crypto::sketches::odd_sketch::*;

use db::rocksdb::RocksDb;
use db::*;
use utils::constants::TX_DB_PATH;

use primitives::status::Status;
use rand::Rng;
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time;
use utils::mining;

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

    let n_mining_threads = 0;

    let mut sketch_bus = Bus::new(10);
    let (distance_send, distance_recv) = channel();

    let (sk, pk) = ecdsa::generate_keypair();

    for i in 0..n_mining_threads {
        let distance_send_c = distance_send.clone();
        let mut sketch_recv = sketch_bus.add_rx();
        thread::spawn(move || {
            mining::mine(
                pk,
                std::u64::MAX * i / (n_mining_threads + 1),
                sketch_recv,
                distance_send_c,
            )
        });
    }

    let status = Arc::new(Status::new(
        RwLock::new(0),
        RwLock::new(Bytes::with_capacity(64)),
        RwLock::new(Bytes::with_capacity(64)),
    ));

    let status_c = status.clone();
    thread::spawn(move || daemon::response_server(tx_db, status_c, pk, sk));

    let sketch_recv = sketch_bus.add_rx();
    thread::spawn(move || status.update_local(sketch_recv, distance_recv));

    let new_tx_interval = time::Duration::from_millis(100);

    loop {
        // state.push(random_tx());
        // sketch_bus.broadcast(state.odd_sketch());
        thread::sleep(new_tx_interval);
    }

    fn random_tx() -> Bytes {
        let mut rng = rand::thread_rng();
        let my_array: [u8; 8] = rng.gen();
        Bytes::from(&my_array[..])
    }
}
