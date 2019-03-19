use bytes::Bytes;
use failure::Error;
use secp256k1::PublicKey;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use futures::sync::mpsc::Sender;
use net::messages::Message;
use primitives::ego::*;
use std::net::SocketAddr;
use utils::byte_ops::Hamming;
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

    pub fn find_leader_sink(&self) -> Option<Sender<Message>> {
        // Lock everything
        let ego_locked = self.ego.lock().unwrap();
        let mut peer_locks: Vec<MutexGuard<PeerEgo>> = self
            .peer_egos
            .iter()
            .map(|(addr, ego)| ego.lock().unwrap())
            .collect();

        // Is a reconcile live?
        if peer_locks
            .iter()
            .any(|ego| ego.get_status() == Status::StatePull)
        {
            None
        } else {
            match peer_locks.pop() {
                None => None,
                Some(leader) => Some(leader.get_sink()),
            }
        }
    }
}
