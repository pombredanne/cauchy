use blake2::{Blake2b, Digest};
use bytes::Bytes;

use crate::utils::constants::HASH_LEN;

pub trait Blk2bHashable: Into<Bytes> {
    fn blake2b(&self) -> Bytes;
}

impl<T: Into<Bytes> + Clone> Blk2bHashable for T {
    fn blake2b(&self) -> Bytes {
        let raw = self.clone().into();
        Bytes::from(&Blake2b::digest(&raw)[..HASH_LEN])
    }
}
