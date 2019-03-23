use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard};

use bytes::Bytes;
use failure::Error;
use futures::sync::mpsc::Sender;
use itertools::Itertools;
use secp256k1::PublicKey;

use net::messages::Message;
use primitives::ego::*;
use utils::constants::ARENA_VERBOSE;
use utils::errors::ArenaError;

pub struct Arena {
    ego: Arc<Mutex<Ego>>,
    peer_egos: HashMap<SocketAddr, Arc<Mutex<PeerEgo>>>,
}

impl Arena {
    pub fn new(ego: Arc<Mutex<Ego>>) -> Arena {
        Arena {
            ego: ego,
            peer_egos: HashMap::new(),
        }
    }

    pub fn new_peer(&mut self, addr: &SocketAddr, peer_ego: Arc<Mutex<PeerEgo>>) {
        self.peer_egos.insert(*addr, peer_ego);
    }

    // // TODO: This seems super dangerous, maybe collect first?
    // pub fn get_state_push_targets(
    // ) -> Vec<SocketAddr> {
    //     peer_egos
    //         .iter()
    //         .filter_map(|(addr, ego)| {
    //             if ego.get_status() == Status::StatePush {
    //                 Some(*addr)
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect()
    // }

    pub fn reconcile_leader(&self) {
        // Lock everything
        let ego_locked = self.ego.lock().unwrap();
        let mut peer_locks: Vec<MutexGuard<PeerEgo>> = self
            .peer_egos
            .iter()
            .map(|(addr, ego)| ego.lock().unwrap())
            .collect();

        // Is a reconcile live?
        if !peer_locks
            .iter()
            .any(|ego| ego.get_status() == Status::StatePull)
        {
            // TODO: Make this faster
            let mut best_distance = 256;
            let mut best_index = 0;
            for (i, guard) in peer_locks.iter().enumerate() {
                let i_distance = peer_locks
                    .iter()
                    .filter_map(|guard_inner| match guard_inner.get_work_site() {
                        Some(work_site) => Some(work_site.mine(&guard.get_oddsketch())),
                        None => None,
                    })
                    .sum();
                if i_distance < best_distance {
                    best_index = i;
                    best_distance = i_distance;
                }
            }

            let self_distance: u16 = peer_locks
                .iter()
                .filter_map(|guard_inner| match guard_inner.get_work_site() {
                    Some(work_site) => Some(work_site.mine(&ego_locked.get_oddsketch())),
                    None => None,
                })
                .sum();

            println!("self distance {}", self_distance);
            println!("best peer distance {}", best_distance);
            if self_distance < best_distance {
                println!("leading");
            } else {
                peer_locks[best_index].update_status(Status::StatePull);
                peer_locks[best_index].send_msg(Message::Reconcile);
            }
        }
    }
}
