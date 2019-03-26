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

    pub fn remove_peer(&mut self, addr: &SocketAddr) {
        self.peer_egos.remove(addr);
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
        let mut participants: Vec<MutexGuard<PeerEgo>> = self
            .peer_egos
            .iter()
            .map(|(_, ego)| ego.lock().unwrap())
            .filter(|guard| {
                guard.get_status() == Status::Gossiping
                    && guard.get_work_status() == WorkStatus::Ready
                    && guard.get_pubkey().is_some()
            })
            .collect();

        // Is a reconcile live?
        if !participants
            .iter()
            .any(|ego| ego.get_status() == Status::StatePull)
        {
            // TODO: Make this faster
            let mut best_distance = 1024;
            let mut best_index = 0;
            for (i, guard) in participants.iter().enumerate() {
                let mut i_distance = participants
                    .iter()
                    .filter_map(|guard_inner| match guard_inner.get_work_site() {
                        Some(work_site) => Some(work_site.mine(&guard.get_oddsketch())),
                        None => None,
                    })
                    .sum();

                i_distance += ego_locked.get_work_site().mine(&guard.get_oddsketch());
                if i_distance < best_distance {
                    best_index = i;
                    best_distance = i_distance;
                }
            }

            let mut self_distance: u16 = participants
                .iter()
                .filter_map(|guard_inner| match guard_inner.get_work_site() {
                    Some(work_site) => Some(work_site.mine(&ego_locked.get_oddsketch())),
                    None => None,
                })
                .sum();
            self_distance += ego_locked.get_current_distance();

            println!("self distance {}", self_distance);
            println!("best peer distance {}", best_distance);
            if self_distance < best_distance {
                println!("leading");
            } else {
                println!("sent reconcile");
                participants[best_index].update_status(Status::StatePull);
                participants[best_index].send_msg(Message::Reconcile);
            }
        }
    }
}
