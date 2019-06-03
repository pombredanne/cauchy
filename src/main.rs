use bus::Bus;

use core::{
    crypto::signatures::ecdsa,
    daemon::{Origin, Priority},
    db::{mongodb::MongoDB, *},
    ego::ego::Ego,
    net::heartbeats::*,
    primitives::{arena::*, tx_pool::TxPool},
    stage::Stage,
    utils::{constants::*, logging::*, mining},
};

use futures::lazy;
use futures::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // std::env::set_var("RUST_LOG", "info");

    // Enviroment logger
    log::set_logger(&CLogger).map(|()| log::set_max_level(log::LevelFilter::Info));

    // Init DB
    let db = MongoDB::open_db("cauchy").unwrap();

    // Generate node key pair
    let (local_sk, local_pk) = ecdsa::generate_keypair();

    // Construct distance pipeline
    let (distance_send, distance_recv) = std::sync::mpsc::channel();
    let mut ego_bus = Bus::new(10);
    let mining_reset = ego_bus.add_rx();

    // Spawn mining threads
    let n_mining_threads: u64 = CONFIG.mining.n_mining_threads as u64;
    if n_mining_threads != 0 {
        let nonce_start_base = std::u64::MAX / n_mining_threads;
        for i in 0..n_mining_threads {
            let distance_send_inner = distance_send.clone();
            let ego_recv = ego_bus.add_rx();

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

    // Init mempool
    let mempool = Arc::new(Mutex::new(TxPool::with_capacity(1024))); // TODO: Make mempool size constant

    // Spawn stage manager
    // let (reset_send, reset_recv) = std::sync::mpsc::channel(); // TODO: Reset mining best
    let (stage_send, stage_recv) = mpsc::channel::<(Origin, TxPool, Priority)>(128);
    let stage = Stage::new(ego.clone(), db.clone(), ego_bus);
    let stage_mananger = stage.manager(mempool.clone(), stage_recv);

    // Server
    let (socket_send, socket_recv) = mpsc::channel::<tokio::net::TcpStream>(128);
    let server = core::daemon::server(
        db.clone(),
        ego.clone(),
        socket_recv,
        arena.clone(),
        mempool.clone(),
        stage_send.clone(),
    );

    // Construct RPC server stack
    let rpc_server_stack = rpc::construct_rpc_stack(socket_send, stage_send, db.clone());

    // Reconciliation heartbeat
    let heartbeat_fut = heartbeat(arena.clone());

    // Spawn servers
    let main_loop = thread::spawn(move || {
        tokio::run(lazy(|| {
            tokio::spawn(stage_mananger);
            tokio::spawn(server);
            for server in rpc_server_stack {
                tokio::spawn(server);
            }
            tokio::spawn(heartbeat_fut);
            Ok(())
        }))
    });

    // Spawn mining updater
    thread::spawn(move || Ego::updater(ego, distance_recv, mining_reset));

    main_loop.join();
}
