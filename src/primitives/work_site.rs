use std::cell::Cell;
use bytes::{Bytes, BytesMut, Buf, BufMut, IntoBuf};
use primitives::varint::VarInt;
use crypto::hashes::blake2b::Blk2bHashable;
use utils::byte_ops::Hamming;

pub struct WorkSite {
	pubkey: Bytes,
	pub nonce: Cell<u64>,
}

impl WorkSite {
	pub fn init(pk: Bytes) -> WorkSite {
		WorkSite{pubkey: pk, nonce: Cell::new(0)}
	}

	pub fn increment(&self) {
		self.nonce.set(self.nonce.get() + 1);
	}

	pub fn set_nonce(&self, nonce: u64) {
		self.nonce.set(nonce);
	}

	pub fn to_bytes(&self) -> Bytes {
		let mut buf = BytesMut::with_capacity(40);
		buf.put(&self.pubkey[..]);
		buf.put_u64_be(self.nonce.get());
		buf.freeze()
	}

	pub fn blake2b(&self) -> Bytes {
		self.to_bytes().blake2b()
	}

	pub fn mine(&self, state_sketch: &Bytes) -> u32 {
		let worksite_b = self.to_bytes().blake2b();
		state_sketch.clone().hamming_distance(worksite_b)
	}
}

impl From<WorkSite> for Bytes {
	fn from(pow: WorkSite) -> Bytes {
		let mut bytes = BytesMut::with_capacity(40);
		bytes.extend_from_slice(&pow.pubkey[..]);
		let vi = VarInt::from(pow.nonce.get());
		bytes.extend_from_slice(&Bytes::from(vi));
		bytes.freeze()
	}
}

impl From<Bytes> for WorkSite {
	fn from(raw: Bytes) -> WorkSite {
		let mut buf = raw.into_buf();
		let pk = &mut [0; 32];
		buf.copy_to_slice(pk);
		let mr = &mut[0; 32];
		buf.copy_to_slice(mr);
		let vi = VarInt::parse(buf.bytes());
		let n = u64::from(vi);
		WorkSite{
			pubkey: Bytes::from(&pk[..]), 
			nonce: Cell::new(n)}
	}
}