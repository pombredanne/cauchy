extern crate blake2;
extern crate bus;
extern crate bytes;
extern crate crossbeam;
extern crate futures;
extern crate rand;
extern crate rocksdb;
extern crate secp256k1;
extern crate tokio;

extern crate core;
extern crate vm;

use bus::Bus;
use bytes::Bytes;
use crossbeam::channel;

use core::{
    crypto::signatures::ecdsa, db::rocksdb::RocksDb, db::storing::Storable, db::*,
    net::heartbeats::*, primitives::arena::*, primitives::ego::*,
    primitives::transaction::Transaction, utils::constants::*, utils::mining,
};
use futures::lazy;
use rand::Rng;
use rocksdb::{Options, DB};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use tokio::sync::mpsc;

fn main() {
    // TODO: Do not destroy DB
    let mut opts = Options::default();
    DB::destroy(&opts, ".cauchy/tests/db_a/");

    let tx_db = Arc::new(RocksDb::open_db(TX_DB_PATH).unwrap());

    let (local_sk, local_pk) = ecdsa::generate_keypair();

    let (distance_send, distance_recv) = channel::unbounded();
    let mut state_proxy_bus = Bus::new(10);
    let n_mining_threads: u64 = CONFIG.MINING.N_MINING_THREADS as u64;
    let nonce_start_base = std::u64::MAX / n_mining_threads;

    for i in 0..n_mining_threads {
        let distance_send_inner = distance_send.clone();
        let mut proxy_recv = state_proxy_bus.add_rx();

        thread::spawn(move || {
            mining::mine(
                local_pk,
                proxy_recv,
                distance_send_inner,
                i * nonce_start_base,
            )
        });
    }

    // Init Ego
    let ego = Arc::new(Mutex::new(Ego::new(local_pk, local_sk)));

    let arena = Arc::new(Mutex::new(Arena::new(ego.clone())));

    // Server
    let (socket_send, socket_recv) = mpsc::channel::<tokio::net::TcpStream>(128);
    let server = core::daemon::server(tx_db.clone(), ego.clone(), socket_recv, arena.clone());

    // RPC Server
    let rpc_server = core::net::rpc_server::rpc_server(socket_send);
    let reconcile_heartbeat = heartbeat_reconcile(arena.clone());

    // Spawn servers
    thread::spawn(move || {
        tokio::run(lazy(|| {
            tokio::spawn(server);
            tokio::spawn(rpc_server);
            tokio::spawn(reconcile_heartbeat);
            Ok(())
        }))
    });

    // Update local state
    let (tx_send, tx_recv) = channel::unbounded();
    // thread::spawn(move || local_status.update_local(state_proxy_bus, tx_recv, distance_recv));

    let new_tx_interval = time::Duration::from_nanos(CONFIG.DEBUGGING.TEST_TX_INTERVAL);

    loop {
        let new_random_tx = random_tx();
        new_random_tx.clone().to_db(tx_db.clone()).unwrap();
        tx_send.send(new_random_tx);
        thread::sleep(new_tx_interval);
    }

    fn random_tx() -> Transaction {
        let mut rng = rand::thread_rng();
        let aux_data: [u8; 8] = rng.gen();
        let binary: [u8; 8] = rng.gen();
        Transaction::new(0, Bytes::from(&aux_data[..]), Bytes::from(&binary[..]))
    }
}
