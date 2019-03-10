use net::connections::ConnectionManager;
use net::messages::Message;
use net::reconcile_status::*;
use primitives::arena::*;
use primitives::status::*;
use secp256k1::PublicKey;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::prelude::*;
use tokio::timer::Interval;
use utils::constants::*;

use failure::Error;
use utils::errors::{HeartBeatNonceError, HeartBeatOddSketchError};

pub fn heartbeat_update(
    arena: Arc<RwLock<Arena>>,
    local_status: Arc<Status>,
    rec_status: Arc<RwLock<ReconciliationStatus>>,
    socket_pk: Arc<RwLock<PublicKey>>,
    socket_addr: SocketAddr,
) -> impl futures::stream::Stream<Item = Message, Error = Error> {
    Interval::new_interval(Duration::new(
        ODDSKETCH_HEARTBEAT_PERIOD_SEC,
        ODDSKETCH_HEARTBEAT_PERIOD_NANO,
    ))
    .map(move |_| *socket_pk.read().unwrap())
    .filter(move |sock_pk| {
        let rec_status_read = rec_status.read().unwrap();
        let live = rec_status_read.is_live();
        let reconcilee = rec_status_read.is_reconcilee(sock_pk);
        !live && !reconcilee
    }) // Wait while reconciling or while sending to reconcilee
    .map(move |sock_pk| {
        (
            local_status.get_total_sketch(),
            local_status.get_nonce(),
            sock_pk,
        )
    })
    .map(move |(current_total_sketch, current_nonce, sock_pk)| {
        let arena_r = &*arena.read().unwrap();
        let perception = arena_r.get_perception(&sock_pk);

        (current_total_sketch, current_nonce, perception)
    })
    .filter(move |(current_total_sketch, current_nonce, perception)| {
        // Check whether peers perception of own nonce needs updating
        match perception {
            Some(some) => {
                (*current_total_sketch != some.get_total_sketch())
                    || (*current_nonce != some.get_nonce())
            } // TODO: Only send nonce if only nonce changes
            None => false,
        }
    })
    .map(move |(current_total_sketch, current_nonce, perception)| {
        if HEARTBEAT_VERBOSE {
            println!("Sending odd sketch to {}", socket_addr);
        }
        // Update perception and send msg
        let perception = perception.unwrap();

        perception.update_total_sketch(&current_total_sketch);
        Message::Update {
            oddsketch: current_total_sketch.oddsketch.clone(),
            root: current_total_sketch.root.clone(),
            nonce: current_nonce,
        }
    })
    .map_err(|_| HeartBeatOddSketchError.into())
}

// TODO: How does this thread die?
// TODO: Clean up
pub fn spawn_heartbeat_reconcile(
    connection_manager: Arc<RwLock<ConnectionManager>>,
    arena: Arc<RwLock<Arena>>,
    rec_status: Arc<RwLock<ReconciliationStatus>>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    let rec_status_inner = rec_status.clone();
    Interval::new_interval(Duration::new(
        RECONCILE_HEARTBEAT_PERIOD_SEC,
        RECONCILE_HEARTBEAT_PERIOD_NANO,
    ))
    .filter(move |_| !rec_status_inner.read().unwrap().is_live()) // Wait while reconciling
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
        let socket_addr = connection_manager_read.get_socket_by_pk(&leader_pk);
        let router_sender = connection_manager_read.get_router_sender();

        (socket_addr, router_sender, leader_pk)
    })
    .filter(move |(socket_addr, _, leader_pk)| {
        if socket_addr.is_some() {
            // Set reconciliation target to leader
            if HEARTBEAT_VERBOSE {
                println!("New reconciliation target: {}", leader_pk);
            }
            let mut rec_status_write_locked = rec_status.write().unwrap();
            rec_status_write_locked.set_target(leader_pk);
            true
        } else {
            if HEARTBEAT_VERBOSE {
                println!("Leader doesn't have an associated socket");
            }
            false
        }
    })
    .for_each(move |(socket_addr, impulse_sender, _)| {
        impulse_sender
            .clone()
            .send((socket_addr.unwrap(), Message::Reconcile))
            // .map_err(|e| ImpulseSendError) // TODO: Capture cause?
            .map(|_| ())
            .or_else(|e| {
                println!("impulse error = {:?}", e);
                Ok(())
            })
    })
    .or_else(|e| {
        println!("error = {:?}", e);
        Ok(())
    })
}
