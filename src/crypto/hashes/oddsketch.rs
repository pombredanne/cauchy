use crypto::hashes::HASH_LEN;
use crypto::hashes::blake2b::Blk2bHashable;
use bytes::Bytes;
use utils::byte_ops::*;


pub trait Sketchable<T>: Into<Vec<T>> where T: Into<Bytes> {
	fn odd_sketch(&self) -> Bytes;
}

pub fn sketched_size(raw: Bytes) -> u32 {
	let n = 8 * raw.len() as u32;
	let z = raw.hamming_weight();
	let n = n as f64;
	let z = z as f64;
	//(- 1. / 2. * f64::ln(1. - 2. * z / n) ) as u32

	(f64::ln(1.-2.* z / n) / f64::ln(1.- 2. / n)) as u32
}

// TODO: Perhaps Murmur 3 instead?
impl<T: Blk2bHashable> Sketchable<T> for Vec<T> {
	fn odd_sketch(&self) -> Bytes {
		let mut pos: u8;
		let mut sketch: [u8; HASH_LEN] = [0; HASH_LEN];
		for value in self {
			pos = value.blake2b().first().unwrap().clone();
			let shift = &pos % 8;
			let index = (&pos / (HASH_LEN >> 3) as u8) as usize;
			sketch[index] = sketch[index] ^ (1 << shift);
		}
		Bytes::from(&sketch[..])
	}
}