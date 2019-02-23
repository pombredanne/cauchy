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
    net::connections::*, net::heartbeats::*, net::reconcile_status::ReconciliationStatus,
    primitives::arena::*, primitives::status::Status, primitives::transaction::Transaction,
    utils::constants::*, utils::mining,
};
use futures::lazy;
use futures::sync::mpsc;
use rand::Rng;
use rocksdb::{Options, DB};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time;

fn main() {
    // TODO: Do not destroy DB
    let mut opts = Options::default();
    DB::destroy(&opts, ".geodesic/tests/db_a/");

    let tx_db = Arc::new(RocksDb::open_db(TX_DB_PATH).unwrap());

    let (local_sk, local_pk) = ecdsa::generate_keypair();

    let (distance_send, distance_recv) = channel::unbounded();
    let mut odd_sketch_bus = Bus::new(10);
    let n_mining_threads: u64 = 1;

    for i in 0..n_mining_threads {
        let distance_send_c = distance_send.clone();
        let mut sketch_recv = odd_sketch_bus.add_rx();

        thread::spawn(move || {
            mining::mine(local_pk, sketch_recv, distance_send_c, i, n_mining_threads)
        });
    }

    let local_status = Arc::new(Status::null());
    let arena = Arc::new(RwLock::new(Arena::init(&local_pk, local_status.clone())));
    let (connection_manager, router) = ConnectionManager::init();
    let recon_status = Arc::new(RwLock::new(ReconciliationStatus::new()));

    // Server
    let (new_socket_tx, new_socket_rx) = mpsc::channel(1);
    let server = core::daemon::server(
        tx_db.clone(),
        local_status.clone(),
        local_pk,
        local_sk,
        new_socket_rx,
        arena.clone(),
        connection_manager.clone(),
        recon_status.clone(),
    );

    // RPC Server
    let rpc_server = core::daemon::rpc_server(new_socket_tx);
    let reconcile_heartbeat =
        spawn_heartbeat_reconcile(connection_manager.clone(), arena.clone(), recon_status);

    // Spawn servers
    thread::spawn(move || {
        tokio::run(lazy(|| {
            tokio::spawn(server);
            tokio::spawn(rpc_server);
            tokio::spawn(reconcile_heartbeat);
            tokio::spawn(router);
            Ok(())
        }))
    });

    // Update local state
    let (sketch_send, sketch_recv) = channel::unbounded();
    thread::spawn(move || local_status.update_local(odd_sketch_bus, sketch_recv, distance_recv));

    let new_tx_interval = time::Duration::from_millis(1000);

    loop {
        let new_random_tx = random_tx();
        new_random_tx.clone().to_db(tx_db.clone()).unwrap();
        sketch_send.send(new_random_tx);
        thread::sleep(new_tx_interval);
    }

    fn random_tx() -> Transaction {
        let mut rng = rand::thread_rng();
        let aux_data: [u8; 8] = rng.gen();
        let binary: [u8; 8] = rng.gen();
        Transaction::new(0, Bytes::from(&aux_data[..]), Bytes::from(&binary[..]))
    }
}