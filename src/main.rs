extern crate blake2;
extern crate bus;
extern crate bytes;
extern crate crossbeam;
extern crate futures;
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
use crossbeam::channel;
use crypto::signatures::ecdsa;
use db::rocksdb::RocksDb;
use db::*;
use futures::lazy;
use futures::sync::mpsc;
use net::connections::*;
use net::heartbeats::*;
use primitives::arena::*;
use primitives::status::Status;
use primitives::transaction::Transaction;
use rand::Rng;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time;
use utils::constants::*;
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

    let (local_sk, local_pk) = ecdsa::generate_keypair();

    let (distance_send, distance_recv) = channel::unbounded();
    let mut odd_sketch_bus = Bus::new(10);
    let n_mining_threads: u64 = 0;

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

    // Server

    let (new_socket_tx, new_socket_rx) = mpsc::channel(1);
    let server = daemon::server(
        tx_db,
        local_status.clone(),
        local_pk,
        local_sk,
        new_socket_rx,
        arena.clone(),
        connection_manager.clone(),
    );

    // RPC Server
    let rpc_server = daemon::rpc_server(new_socket_tx);
    let reconcile_heartbeat = spawn_heartbeat_reconcile(connection_manager.clone(), arena.clone());

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

    let new_tx_interval = time::Duration::from_millis(2000);

    loop {
        sketch_send.send(random_tx());
        thread::sleep(new_tx_interval);
    }

    fn random_tx() -> Transaction {
        let mut rng = rand::thread_rng();
        let aux_data: [u8; 8] = rng.gen();
        let binary: [u8; 8] = rng.gen();
        Transaction::new(0, Bytes::from(&aux_data[..]), Bytes::from(&binary[..]))
    }
}
