use crypto::hashes::HASH_LEN;
use crypto::hashes::blake2b::Blk2bHashable;
use bytes::Bytes;
use utils::byte_ops::*;

pub struct Sketch(Bytes);

impl Sketch {
	pub fn size(&self) -> u32 {
		let n = self.0.len() as f64;
		let z = self.0.hamming_weight() as f64;
		(f64::ln(1.-2.* z / n) / f64::ln(1.- 2. / n)) as u32
	}

	pub fn xor(self, rhs: Sketch) -> Sketch {
		Sketch(Bytes::byte_xor(self.into(), rhs.into()))
	}
}

impl From<Bytes> for Sketch {fn from(raw: Bytes) -> Sketch { Sketch(raw) }}

impl From<Sketch> for Bytes {fn from(sketch: Sketch) -> Bytes { sketch.0 }}

pub trait Sketchable<T>: Into<Vec<T>> where T: Into<Bytes> {
	fn odd_sketch(&self) -> Sketch;
}

// TODO: Perhaps Murmur 3 instead?
impl<T: Blk2bHashable> Sketchable<T> for Vec<T> {
	fn odd_sketch(&self) -> Sketch {
		let mut pos: u8;
		let mut sketch: [u8; HASH_LEN] = [0; HASH_LEN];
		for value in self {
			pos = value.blake2b().first().unwrap().clone();
			let shift = &pos % 8;
			let index = (&pos / (HASH_LEN >> 3) as u8) as usize;
			sketch[index] = sketch[index] ^ (1 << shift);
		}
		Sketch(Bytes::from(&sketch[..]))
	}
}

