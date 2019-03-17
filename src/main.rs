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
use rand::Rng;
use rocksdb::{Options, DB};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time;

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

    let local_status = Arc::new(Status::null());
    let arena = Arc::new(RwLock::new(Arena::init(&local_pk, local_status.clone())));

    // Initialise connection manager
    let (connection_manager, new_socket_rx, router) = ConnectionManager::init();

    // Initialise reconciliation status
    let recon_status = Arc::new(RwLock::new(ReconciliationStatus::new()));

    // Server
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
    let rpc_server = core::net::rpc_server::rpc_server(connection_manager.clone());
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
    let (tx_send, tx_recv) = channel::unbounded();
    thread::spawn(move || local_status.update_local(state_proxy_bus, tx_recv, distance_recv));

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
