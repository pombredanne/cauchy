use net::connections::ConnectionManager;
use net::messages::Message;
use primitives::arena::*;
use primitives::status::*;
use secp256k1::PublicKey;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::io::{Error, ErrorKind};
use tokio::prelude::*;
use tokio::timer::Interval;
use utils::constants::*;

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
    .map(move |_| {
        (
            local_status.get_odd_sketch(),
            local_status.get_mini_sketch(),
            *socket_pk.read().unwrap(),
        )
    })
    .map(move |(current_odd_sketch, current_mini_sketch, sock_pk)| {
        let arena_r = &*arena.read().unwrap();
        let perception = arena_r.get_perception(&sock_pk);

        (current_odd_sketch, current_mini_sketch, perception)
    })
    .filter(move |(current_odd_sketch, _, perception)| {
        // Check whether peers perception of own nonce needs updating
        match perception {
            Some(some) => *current_odd_sketch != some.get_odd_sketch(),
            None => false,
        }
    })
    .map(
        move |(current_odd_sketch, current_mini_sketch, perception)| {
            if VERBOSE {
                println!("Sending odd sketch to {}", socket_addr);
            }
            // Update perception and send msg
            let perception = perception.unwrap();
            perception.update_odd_sketch(current_odd_sketch.clone());
            perception.update_mini_sketch(current_mini_sketch);
            Message::OddSketch {
                sketch: current_odd_sketch,
            }
        },
    )
    .map_err(|_| Error::new(ErrorKind::Other, "Odd sketch heart failure"))
}

pub fn heartbeat_nonce(
    arena: Arc<RwLock<Arena>>,
    local_status: Arc<Status>,
    socket_pk: Arc<RwLock<PublicKey>>,
    dummy_pk: PublicKey, // TODO: This shouldn't be the condition (it should be perceived pk)
    socket_addr: SocketAddr,
) -> impl futures::stream::Stream<Item = Message, Error = Error> {
    Interval::new_interval(Duration::new(
        NONCE_HEARTBEAT_PERIOD_SEC,
        NONCE_HEARTBEAT_PERIOD_NANO,
    ))
    .map(move |_| (local_status.get_nonce(), *socket_pk.read().unwrap()))
    .filter(move |(_, sock_pk)| *sock_pk != dummy_pk)
    .map(move |(current_nonce, sock_pk)| {
        let arena_r = &*arena.read().unwrap();
        let perception = arena_r.get_perception(&sock_pk);

        (current_nonce, perception)
    })
    .filter(move |(current_nonce, perception)| {
        // Check whether peers perception of own nonce needs updating
        match perception {
            Some(some) => *current_nonce != some.get_nonce(),
            None => false,
        }
    })
    .map(move |(current_nonce, perception)| {
        if VERBOSE {
            println!("Sending nonce to {}", socket_addr);
        }

        // Update perception and send msg
        perception.unwrap().update_nonce(current_nonce);
        Message::Nonce {
            nonce: current_nonce,
        }
    })
    .map_err(|_| Error::new(ErrorKind::Other, "Nonce heart failure"))
}

// TODO: How does this thread die?
// TODO: Clean up
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

        let connection_manager_inner = connection_manager.clone();
        let connection_manager_read = &*connection_manager_inner.read().unwrap();
        let socket_addr = connection_manager_read.get_socket_by_pk(leader_pk); // TODO: This is super unsafe
        let router_sender = connection_manager_read.get_router_sender();

        (socket_addr, router_sender)
    })
    .filter(move |(socket_addr, _)| socket_addr.is_some())
    .for_each(move |(socket_addr, router_sender)| {
        router_sender
            .clone()
            .send((socket_addr.unwrap(), Message::Reconcile))
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
