use std::collections::HashSet;

use bus::Bus;
use bytes::Bytes;

use ::rocksdb::{Options, DB};
use core::{
    crypto::signatures::ecdsa,
    db::rocksdb::RocksDb,
    db::storing::Storable,
    db::*,
    net::heartbeats::*,
    primitives::arena::*,
    primitives::ego::{Ego, PeerEgo},
    primitives::transaction::Transaction,
    utils::constants::*,
    utils::mining,
    utils::timing::*,
};
use futures::lazy;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use tokio::sync::mpsc;

fn main() {
    // TODO: Do not destroy DB
    let mut opts = Options::default();
    DB::destroy(&opts, TX_DB_PATH);

    let tx_db = Arc::new(RocksDb::open_db(TX_DB_PATH).unwrap());

    let (local_sk, local_pk) = ecdsa::generate_keypair();

    let (distance_send, distance_recv) = std::sync::mpsc::channel();
    let mut ego_bus = Bus::new(10);

    // Spawn mining threads
    let n_mining_threads: u64 = CONFIG.MINING.N_MINING_THREADS as u64;
    if n_mining_threads != 0 {
        let nonce_start_base = std::u64::MAX / n_mining_threads;
        for i in 0..n_mining_threads {
            let distance_send_inner = distance_send.clone();
            let mut ego_recv = ego_bus.add_rx();

            thread::spawn(move || {
                mining::mine(
                    local_pk,
                    ego_recv,
                    distance_send_inner,
                    i * nonce_start_base,
                )
            });
        }
    }

    // Init Ego
    let ego = Arc::new(Mutex::new(Ego::new(local_pk, local_sk)));

    // Init Arena
    let arena = Arc::new(Mutex::new(Arena::new(ego.clone())));

    // Spawn stage manager
    let (reset_send, reset_recv) = std::sync::mpsc::channel();
    let (to_stage, stage_recv) = mpsc::channel::<(Arc<Mutex<PeerEgo>>, HashSet<Transaction>)>(128); // Send incoming

    // Server
    let (socket_send, socket_recv) = mpsc::channel::<tokio::net::TcpStream>(128);
    let server = core::daemon::server(
        tx_db.clone(),
        ego.clone(),
        socket_recv,
        arena.clone(),
        to_stage,
    );

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
    thread::spawn(move || Ego::mining_updater(ego, distance_recv, reset_recv));

    let new_tx_interval = duration_from_millis(CONFIG.DEBUGGING.TEST_TX_INTERVAL);

    loop {
        let new_random_tx = random_tx();
        new_random_tx.clone().to_db(tx_db.clone()).unwrap();
        thread::sleep(new_tx_interval);
    }

    fn random_tx() -> Transaction {
        let mut rng = rand::thread_rng();
        let aux_data: [u8; 8] = rng.gen();
        let binary: [u8; 8] = rng.gen();
        Transaction::new(0, Bytes::from(&aux_data[..]), Bytes::from(&binary[..]))
    }
}
