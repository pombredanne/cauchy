use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::ops::DerefMut;

use log::info;
use rand::seq::SliceRandom;
use secp256k1::PublicKey;

use crate::{
    crypto::sketches::odd_sketch::OddSketch,
    ego::{ego::*, peer_ego::*, *},
    net::{messages::Message, peers::Peer},
    primitives::{
        status::*,
        work::{WorkStack, WorkState, WorkSite},
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

    pub fn get_ego(&self) -> Arc<Mutex<Ego>> {
        self.ego.clone()
    }

    pub fn new_peer(&mut self, addr: &SocketAddr, peer_ego: Arc<Mutex<PeerEgo>>) {
        arena_info!("added {} to arena", addr);
        self.peer_egos.insert(*addr, peer_ego);
    }

    pub fn remove_peer(&mut self, addr: &SocketAddr) {
        arena_info!("removed {} from arena", addr);
        self.peer_egos.remove(addr);
    }

    pub fn work_pulse(&self, size: usize) {
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
            .take(size) // TODO: Shuffle before taking
            .for_each(|mut peer_ego_guard| {
                peer_ego_guard.update_status(PeerStatus::WorkPull);
                peer_ego_guard.send_msg(Message::GetWork);
            })
    }

    pub fn reconcile_leader(&self) {
        // Lock self
        let mut ego_guard = self.ego.lock().unwrap();

        // Lock fighters
        let mut profiles: Vec<(MutexGuard<PeerEgo>, WorkStack, PublicKey)> = self.peer_egos.values().filter_map(|peer_ego| {
            let peer_ego_guard = peer_ego.lock().unwrap();
            match (peer_ego_guard.get_status(), peer_ego_guard.get_pubkey()) {
                (PeerStatus::Fighting(work_state), Some(public_key)) => {
                    Some((peer_ego_guard, work_state, public_key))
                }
                _ => None,
            }
        }).collect();
        
        let mut best_peer = None;
        let mut best_dist = 0;

        // Calculate own distance
        for (_, work_stack_inner, pubkey_inner) in &profiles {
            let work_site = WorkSite::new(*pubkey_inner, work_stack_inner.get_root(), work_stack_inner.get_nonce());
            best_dist += work_site.mine(ego_guard.work_stack.get_oddsketch())
        }

        best_dist += ego_guard.get_work_site().mine(ego_guard.work_stack.get_oddsketch());

        arena_info!("self distance: {}", best_dist);

        // Calculate peer distance
        for (i, (_, work_stack, _)) in profiles.iter().enumerate() {
            let mut dist = 0;
            for (_, work_stack_inner, pubkey_inner) in &profiles {
                let work_site_inner = WorkSite::new(*pubkey_inner, work_stack_inner.get_root(), work_stack_inner.get_nonce());
                dist += work_site_inner.mine(work_stack.get_oddsketch())
            }

            dist += ego_guard.get_work_site().mine(work_stack.get_oddsketch());
            arena_info!("peer distance: {}", dist);
            if dist <= best_dist {
                best_dist = dist;
                best_peer = Some(i);
            }
        }

        match best_peer {
            Some(i) => {
                for (j, (peer_ego, work_stack, _)) in profiles.iter_mut().enumerate() {
                    if i == j {
                        // Update status to pulling with expectation grabbed from current status
                        let expectation = Expectation::new(work_stack.get_oddsketch(), work_stack.get_root());
                        peer_ego.update_status(PeerStatus::StatePull(expectation));
                        ego_guard.update_status(Status::Pulling);

                        // Send reconcile message
                        peer_ego.send_msg(Message::Reconcile);
                    } else {
                        // Reset to losers to idle
                        peer_ego.update_status(PeerStatus::Idle);
                    }
                }
                },
            None => {
                // Leading
                arena_info!("leading");

                // Reset losers to idle
                for (peer_ego,_, _) in profiles.iter_mut() {
                    peer_ego.update_status(PeerStatus::Idle);
                }
            }
        }
    }
}
