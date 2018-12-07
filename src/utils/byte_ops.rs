extern crate bytes;
use bytes::{Bytes, Buf, BufMut, IntoBuf};
use std::ops::*;


macro_rules! bop {
	($trait_name: ident, $fn_name: ident) => (
		pub trait $trait_name {

			fn $fn_name(self, rhs: Bytes) -> Bytes;
		}

		impl $trait_name for Bytes {

			fn $fn_name(self, rhs: Self) -> Bytes {
				let len = self.len();
				let mut result = Vec::with_capacity(self.len());
				let mut buf_lhs = self.into_buf();
				let mut buf_rhs = rhs.into_buf();

				for _i in 0..len {
					let x = buf_lhs.get_u8();
					let y = buf_rhs.get_u8();
					result.put(u8::$fn_name(x, y));
				}
				Bytes::from(result)
			}
		}
	)
}

bop!(BitAndByte, bitand);
bop!(BitOrByte, bitor);
bop!(BitXorByte, bitxor);

pub trait Hamming {
	fn hamming_weight(&self) -> u32;
	fn hamming_distance(self, Bytes) -> u32;
}

impl Hamming for Bytes {
	fn hamming_weight(&self) -> u32 {
		let len = self.len();

		let mut count = 0;
		let mut buf = self.into_buf();

		let mut current: u8;
		for _i in 0..len {
			current = buf.get_u8();
			count += current.count_ones();
		}

		count
	}

	fn hamming_distance(self, rhs: Bytes) -> u32 {
		Bytes::bitxor(self, rhs).hamming_weight()
	}
}