use blake2::{Blake2b, Digest};
use bytes::Bytes;

pub trait Blk2bHashable: Into<Bytes> {
    fn blake2b(&self) -> Bytes;
}

impl<T: Into<Bytes> + Clone> Blk2bHashable for T {
    fn blake2b(&self) -> Bytes {
        let raw = self.clone().into();
        let digest = Bytes::from(&Blake2b::digest(&raw)[..]);
        digest
    }
}
