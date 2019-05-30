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

    pub fn reconcile_leader(&self) {
        // Lock self
        let ego_guard = self.ego.lock().unwrap();

        // Lock fighters
        let mut profiles: Vec<(MutexGuard<PeerEgo>, WorkStack, PublicKey)> = self.peer_egos.values().filter_map(|peer_ego| {
            let peer_ego_guard = peer_ego.lock().unwrap();
            match (peer_ego_guard.get_status(), peer_ego_guard.get_pubkey()) {
                (Status::Fighting(work_state), Some(public_key)) => {
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

        // Calculate peer distance
        for (i, (_, work_stack, _)) in profiles.iter().enumerate() {
            let mut dist = 0;
            for (_, work_stack_inner, pubkey_inner) in &profiles {
                let work_site = WorkSite::new(*pubkey_inner, work_stack_inner.get_root(), work_stack_inner.get_nonce());
                dist += work_site.mine(work_stack.get_oddsketch())
            }
            if dist < best_dist {
                best_dist = dist;
                best_peer = Some(i);
            }
        }

        match best_peer {
            Some(i) => {
                let (peer_ego, work_stack, _) = profiles.get_mut(i).unwrap();
                // Update status to State pull with expectation grabbed from
                let expectation = Expectation::new(work_stack.get_oddsketch(), work_stack.get_root());
                peer_ego.update_status(Status::StatePull(expectation));

                // Send reconcile message
                peer_ego.send_msg(Message::Reconcile);
                },
            None => {
                // Leading
                arena_info!("leading")
            }
        }


    }
}
