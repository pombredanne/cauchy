use std::{collections::HashSet, net::SocketAddr};

use rand::seq::SliceRandom;

#[derive(Clone)]
enum Liveness {
    Live,
    Dormant,
}

#[derive(Clone)]
pub struct Peer {
    liveness: Liveness,
    addr: SocketAddr,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Peer {
        Peer {
            liveness: Liveness::Live,
            addr,
        }
    }
    pub fn get_addr(&self) -> SocketAddr {
        self.addr
    }
}

pub struct Peers(Vec<Peer>);

impl Peers {
    pub fn new(peers: Vec<Peer>) -> Peers {
        Peers(peers)
    }

    pub fn insert_batch(&mut self, new_peers: &mut Vec<Peer>) {
        self.0.append(new_peers);
    }

    pub fn sample(&self) -> Option<Peer> {
        let mut rng = &mut rand::thread_rng();
        self.0.choose(&mut rng).cloned()
    }

    pub fn choose_n(&self, n: usize) -> Vec<Peer> {
        let mut rng = &mut rand::thread_rng();
        self.0.choose_multiple(&mut rng, n).cloned().collect()
    }

    pub fn to_vec(self) -> Vec<Peer> {
        self.0
    }
}
