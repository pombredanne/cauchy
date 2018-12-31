use bytes::Bytes;
use crypto::hashes::blake2b::Blk2bHashable;
use crypto::util;
use utils::byte_ops::*;
use utils::constants::SKETCH_LEN;

pub trait Sketchable<T>: Into<Vec<T>>
where
    T: Into<Bytes>,
{
    fn odd_sketch(&self) -> Bytes;
}

pub fn add_to_bin<T>(sketch: &mut [u8], item: &T)
where
    T: Blk2bHashable,
{
    let (shift, index) = util::get_bit_pos(item, SKETCH_LEN);
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

// TODO: Perhaps Murmur 3 instead?
impl<T: Blk2bHashable> Sketchable<T> for Vec<T> {
    fn odd_sketch(&self) -> Bytes {
        let mut sketch: [u8; SKETCH_LEN] = [0; SKETCH_LEN];
        for item in self {
            let (shift, index) = util::get_bit_pos(item, SKETCH_LEN);
            sketch[index] ^= 1 << shift;
        }
        Bytes::from(&sketch[..])
    }
}
