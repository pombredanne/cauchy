use consensus::peer::Peer;
use utils::byte_ops::*;

struct Arena {
    peers: Vec<Peer>,
}

impl Arena {
    pub fn get_distances(&self) -> Vec<u32> {
        // TODO: This can be change as a filter out non-ready futures?
        // TODO: Do this more neatly
        let sketches = self.peers.iter().map(|x| x.get_state_sketch());
        let mut digests = self.peers.iter().map(|x| x.get_work_digest());

        let mut distances = Vec::with_capacity(digests.len());
        for sketch in sketches {
            let mut dist = 0;
            for digest in digests.by_ref() {
                dist += sketch.clone().hamming_distance(digest);
            }
            distances.push(dist);
        }
        distances
    }

    pub fn get_leader(&self) -> Option<&Peer> {
        let pos = match self
            .get_distances()
            .iter()
            .enumerate()
            .max_by_key(|(_, &y)| y)
        {
            Some((pos, _)) => pos,
            None => return None,
        };
        self.peers.get(pos)
    }
}
