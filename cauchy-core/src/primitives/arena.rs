use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::{net::messages::Message, primitives::ego::*};

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

    pub fn reconcile_leader(&self) {
        // Lock everything
        let ego_guard = self.ego.lock().unwrap();
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
            let mut best_distance = 1024;
            let mut best_index = 0;
            for (i, guard) in participants.iter().enumerate() {
                let oddsketch = guard.get_oddsketch();
                let distance = 0;
                for guard_inner in participants.iter() {
                    match guard_inner.get_work_site() {
                        Some(work_site) => {
                            distance += work_site.mine(&oddsketch);
                            if distance > best_distance {
                                break
                            }
                        }, 
                        None => ()
                    }
                }
                if distance < best_distance {
                    best_distance = distance;
                    best_index = i;
                }
            }

            let mut self_distance: u16 = participants
                .iter()
                .map(|guard_inner| ego_guard.get_work_site().mine(&guard_inner.get_oddsketch()))
                .sum(); // TODO: Should we filter non-miners here?
            self_distance += ego_guard.get_current_distance();

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
