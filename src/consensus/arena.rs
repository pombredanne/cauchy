use consensus::status::Status;
use utils::byte_ops::*;

struct Arena {
    peer_status: Vec<Status>,
}

impl Arena {
    pub fn get_distances(&self) -> Vec<u32> {
        // TODO: This can be change as a filter out non-ready futures?
        // TODO: Do this more neatly
        let sketches = self.peer_status.iter().map(|x| x.get_state_sketch());
        let digests = self.peer_status.iter().map(|x| x.get_site_hash());

        let mut distances = Vec::with_capacity(digests.len());
        for sketch in sketches {
            let mut dist = 0;
            for digest in digests.clone() {
                dist += sketch.clone().hamming_distance(digest);
            }
            distances.push(dist);
        }
        distances
    }

    pub fn get_leader(&self) -> Option<&Status> {
        let pos = match self
            .get_distances()
            .iter()
            .enumerate()
            .min_by_key(|(_, &y)| y)
        {
            Some((pos, _)) => pos,
            None => return None,
        };
        self.peer_status.get(pos)
    }
}
