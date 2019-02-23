use bytes::{Bytes, BytesMut};
use crypto::hashes::blake2b::Blk2bHashable;
use crypto::sketches::*;
use crypto::util;
use utils::byte_ops::*;
use utils::constants::SKETCH_CAPACITY;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OddSketch(pub BytesMut);

impl SketchInsertable for OddSketch {
    fn new() -> OddSketch {
        OddSketch(BytesMut::from(&[0; SKETCH_CAPACITY][..]))
    }
    fn insert<T>(&mut self, item: &T)
    where
        T: Blk2bHashable,
    {
        let (shift, index) = util::get_bit_pos(item, SKETCH_CAPACITY);
        self.0[index] ^= 1 << shift;
    }

    fn insert_id(&mut self, item: &Bytes) {
        let (shift, index) = util::get_id_bit_pos(item, SKETCH_CAPACITY);
        self.0[index] ^= 1 << shift;
    }
}

impl OddSketch {
    pub fn size(&self) -> u32 {
        let n = 8 * self.0.len() as u32;
        let z = self.0.clone().freeze().hamming_weight();
        let n = f64::from(n);
        let z = f64::from(z);
        //(-  f64::ln(1. - 2. * z / n) / 2) as u32

        (f64::ln(1. - 2. * z / n) / f64::ln(1. - 2. / n)) as u32
    }

    pub fn xor(&self, other: &OddSketch) -> OddSketch {
        OddSketch(BytesMut::from(
            Bytes::from(self.0.clone()).byte_xor(Bytes::from(other.0.clone())),
        ))
    } // TODO: This is super clunky, rework byte ops
}

impl From<OddSketch> for Bytes {
    fn from(sketch: OddSketch) -> Bytes {
        sketch.0.freeze()
    }
}

impl<T> From<T> for OddSketch
where
    T: Into<BytesMut>,
{
    fn from(raw: T) -> OddSketch {
        OddSketch(raw.into())
    }
}
