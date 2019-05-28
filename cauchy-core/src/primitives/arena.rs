use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard};

use log::info;
use rand::seq::SliceRandom;

use crate::{
    crypto::sketches::odd_sketch::OddSketch,
    ego::{ego::*, peer_ego::*, *},
    net::{messages::Message, peers::Peer},
    primitives::{
        status::*,
        work::{WorkStack, WorkState},
    },
    utils::constants::CONFIG,
};

macro_rules! arena_info {
    ($($arg:tt)*) => {
        if CONFIG.debugging.arena_verbose {
            info!(target: "arena_event", $($arg)*);
        }
    };
}

pub struct Arena {
    ego: Arc<Mutex<Ego>>,
    peer_egos: HashMap<SocketAddr, Arc<Mutex<PeerEgo>>>,
}

impl Arena {
    pub fn new(ego: Arc<Mutex<Ego>>) -> Arena {
        Arena {
            ego,
            peer_egos: HashMap::new(),
        }
    }

    pub fn new_peer(&mut self, addr: &SocketAddr, peer_ego: Arc<Mutex<PeerEgo>>) {
        self.peer_egos.insert(*addr, peer_ego);
    }

    pub fn remove_peer(&mut self, addr: &SocketAddr) {
        self.peer_egos.remove(addr);
    }

    pub fn work_pulse(&mut self, size: usize) {
        self.peer_egos
            .values()
            .filter_map(|peer_ego| {
                // Select from only those with a public key
                let peer_ego_guard = peer_ego.lock().unwrap();
                if peer_ego_guard.get_pubkey().is_some() {
                    Some(peer_ego_guard)
                } else {
                    None
                }
            })
            .take(size)
            .for_each(|mut peer_ego_guard| {
                peer_ego_guard.update_status(Status::WorkPull);
                peer_ego_guard.send_msg(Message::GetWork);
            })
    }

    // pub fn reconcile_leader(&self) {
    //     // Lock everything
    //     let ego_guard = self.ego.lock().unwrap();
    //     let mut participants: Vec<MutexGuard<PeerEgo>> = self
    //         .peer_egos
    //         .values()
    //         .filter_map(|peer_ego| {
    //             // Select from only those fighting
    //             let peer_ego_guard = peer_ego.lock().unwrap();
    //             if peer_ego_guard.get_status() == Status::Fighting {
    //                 Some(peer_ego_guard)
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect();

    //     // Is a reconcile live?
    //     if !participants
    //         .iter()
    //         .any(|ego| ego.get_status() == Status::StatePull)
    //     {
    //         let mut best_distance = 1024;
    //         let mut best_index = 0;
    //         for (i, guard) in participants.iter().enumerate() {
    //             let oddsketch: OddSketch = Default::default();
    //             let mut distance = 0;
    //             // TODO: Locking all peers could be avoided by moving state into the key of hashmap
    //             for guard_inner in participants.iter() {
    //                 if let Some(work_site) = guard_inner.get_work_site() {
    //                     distance += work_site.mine(&oddsketch);
    //                     if distance > best_distance {
    //                         break;
    //                     }
    //                 }
    //             }
    //             if distance < best_distance {
    //                 best_distance = distance;
    //                 best_index = i;
    //             }
    //         }

    //         let mut self_distance: u16 = participants
    //             .iter()
    //             .map(|guard_inner| ego_guard.get_work_site().mine(&guard_inner.get_oddsketch()))
    //             .sum(); // TODO: Should we filter non-miners here?
    //         self_distance += ego_guard.get_current_distance();

    //         arena_info!("self distance {}", self_distance);
    //         arena_info!("best peer distance {}", best_distance);
    //         if self_distance < best_distance {
    //             arena_info!("leading");
    //         } else {
    //             arena_info!("sent reconcile");
    //             participants[best_index].update_status(Status::StatePull);
    //             participants[best_index].send_msg(Message::Reconcile);
    //         }
    //     }
    // }

    pub fn reconcile_leader(&self) {
        let ego_guard = self.ego.lock().unwrap();
        let profiles = self.peer_egos.iter().filter_map(|(addr, peer_ego)| {
            let peer_ego_guard = peer_ego.lock().unwrap();
            match (peer_ego_guard.get_status(), peer_ego_guard.get_pubkey()) {
                (Status::Fighting(work_state), Some(public_key)) => {
                    Some((Some(addr), work_state, public_key))
                }
                _ => None,
            }
        });

        // profiles.map(|(Some(addr), work_state, public_key)| {
        //     profiles.map(|(Some(addr), work_state, public_key)| 0).sum()
        // });
    }
}
