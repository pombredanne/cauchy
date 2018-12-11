use bytes::Bytes;
use crypto::hashes::blake2b::Blk2bHashable;


pub struct PoW {
	pubkey: Bytes,
	nonce: u64,
	state_sketch: Bytes,
}

impl PoW {
	pub fn to_shifted(&self) -> Bytes {
		let mut buf = BytesMut::with_capacity(40);
		buf.put(&self.pubkey[..]);
		buf.put_u64_be(self.nonce.get());
		buf.freeze().blake2b().byte_or(state_sketch)
	}
}