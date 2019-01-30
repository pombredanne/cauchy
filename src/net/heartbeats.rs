use futures::sync::mpsc::Sender;
use net::messages::Message;
use primitives::arena::*;
use primitives::status::*;
use secp256k1::{PublicKey, SecretKey};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::io::{Error, ErrorKind};
use tokio::prelude::*;
use tokio::timer::Interval;
use utils::constants::*;
use net::connections::ConnectionManager;

// TODO: Handle errors properly

pub fn heartbeat_oddsketch(
    arena: Arc<RwLock<Arena>>,
    local_status: Arc<Status>,
    socket_pk: Arc<RwLock<PublicKey>>,
    socket_addr: SocketAddr,
) -> impl futures::stream::Stream<Item = Message, Error = Error> {
    Interval::new_interval(Duration::new(
        ODDSKETCH_HEARTBEAT_PERIOD_SEC,
        ODDSKETCH_HEARTBEAT_PERIOD_NANO,
    ))
    .map(move |_| (local_status.get_odd_sketch(), *socket_pk.read().unwrap()))
    .filter(move |(current_sketch, sock_pk)| {
        // Check whether peers perception of own Odd Sketch needs updating
        match (*arena.read().unwrap()).get_perception(sock_pk) {
            Some(perception) => (*current_sketch != perception.odd_sketch),
            None => false,
        }
    })
    .map(move |(current_sketch, _)| {
        if VERBOSE {
            println!("Sending odd sketch to {}", socket_addr);
        }

        Message::OddSketch {
            sketch: current_sketch,
        }
    })
    .map_err(|_| Error::new(ErrorKind::Other, "Odd sketch heart failure"))
}

pub fn heartbeat_nonce(
    arena: Arc<RwLock<Arena>>,
    local_status: Arc<Status>,
    socket_pk: Arc<RwLock<PublicKey>>,
    dummy_pk: PublicKey,
    socket_addr: SocketAddr,
) -> impl futures::stream::Stream<Item = Message, Error = Error> {
    let arena_inner = arena.clone();
    Interval::new_interval(Duration::new(
        NONCE_HEARTBEAT_PERIOD_SEC,
        NONCE_HEARTBEAT_PERIOD_NANO,
    ))
    .map(move |_| (local_status.get_nonce(), *socket_pk.read().unwrap()))
    .filter(move |(_, sock_pk)| *sock_pk != dummy_pk)
    .filter(move |(current_nonce, sock_pk)| {
        // Check whether peers perception of own nonce needs updating
        match (*arena_inner.read().unwrap()).get_perception(sock_pk) {
            Some(perception) => (*current_nonce != perception.nonce),
            None => false,
        }
    })
    .map(move |(current_nonce, sock_pk)| {
        if VERBOSE {
            println!("Sending nonce to {}", socket_addr);
        }
        let mut arena_w = arena.write().unwrap();
        (*arena_w).update_perception(&sock_pk);
        drop(arena_w);

        Message::Nonce {
            nonce: current_nonce,
        }
    })
    .map_err(|_| Error::new(ErrorKind::Other, "Nonce heart failure"))
}

// TODO: How does this thread die?
pub fn spawn_heartbeat_reconcile(
    connection_manager: Arc<RwLock<ConnectionManager>>,
    arena: Arc<RwLock<Arena>>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    Interval::new_interval(Duration::new(
        RECONCILE_HEARTBEAT_PERIOD_SEC,
        RECONCILE_HEARTBEAT_PERIOD_NANO,
    ))
    .map(move |_| {
        // Update order
        let mut arena_r = arena.write().unwrap();
        (*arena_r).update_order();
        drop(arena_r);

        // Find leader
        let arena_r = arena.read().unwrap();
        let leader_pk = arena_r.get_order()[0];
        drop(arena_r);
        leader_pk
    })
    .for_each(move |leader_pk| {
        let connection_manager_inner = connection_manager.clone();
        let connection_manager_read = &*connection_manager_inner.read().unwrap();
        let socket_addr = connection_manager_read.get_socket_by_pk(leader_pk).unwrap(); // TODO: This is super unsafe
        let router_sender = connection_manager_read.get_router_sender();
        router_sender.clone()
            .send((socket_addr, Message::Reconcile))
            .map_err(|e| Error::new(ErrorKind::Other, "RPC addpeer channel failure"))
            .map(|_| ())
            .or_else(|e| {
                println!("error = {:?}", e);
                Ok(())
            })
    })
    .or_else(|e| {
        println!("error = {:?}", e);
        Ok(())
    })
}
