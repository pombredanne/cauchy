use bytes::{Bytes, BytesMut};
use crypto::hashes::blake2b::Blk2bHashable;
use crypto::util;
use std::iter::IntoIterator;
use utils::byte_ops::*;
use utils::constants::SKETCH_CAPACITY;

pub trait Sketchable {
    fn odd_sketch(&self) -> Bytes;
}

pub fn add_to_bin<T>(sketch: &mut BytesMut, item: &T)
where
    T: Blk2bHashable,
{
    let (shift, index) = util::get_bit_pos(item, SKETCH_CAPACITY);
    sketch[index] ^= 1 << shift;
}

pub fn sketched_size(raw: &Bytes) -> u32 {
    let n = 8 * raw.len() as u32;
    let z = raw.hamming_weight();
    let n = f64::from(n);
    let z = f64::from(z);
    //(-  f64::ln(1. - 2. * z / n) / 2) as u32

    (f64::ln(1. - 2. * z / n) / f64::ln(1. - 2. / n)) as u32
}

impl<T: Blk2bHashable, U> Sketchable for U
where
    U: IntoIterator<Item = T>,
    U: Clone,
{
    fn odd_sketch(&self) -> Bytes {
        let mut sketch: [u8; SKETCH_CAPACITY] = [0; SKETCH_CAPACITY];
        for item in self.clone().into_iter() {
            let (shift, index) = util::get_bit_pos(&item, SKETCH_CAPACITY);
            sketch[index] ^= 1 << shift;
        }
        Bytes::from(&sketch[..])
    }
}
