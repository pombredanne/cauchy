pub mod blake2b;

use bytes::Bytes;

pub trait Identifiable {
    fn get_id(&self) -> Bytes;
}
