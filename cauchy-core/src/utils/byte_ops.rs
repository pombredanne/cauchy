extern crate bytes;

use bytes::{Buf, Bytes, IntoBuf};
use std::ops::*;

macro_rules! bytes {
    ($val: ident) => {
        Bytes::from(&b$val[..])
    };
}

macro_rules! bop {
    ($trait_name: ident, $fn_name: ident, $bitop_name: ident) => {
        pub trait $trait_name {
            fn $fn_name(self, rhs: Bytes) -> Bytes;
        }

        impl $trait_name for Bytes {
            fn $fn_name(self, rhs: Self) -> Bytes {
                let buf_lhs = self.into_buf();
                let buf_rhs = rhs.into_buf();
                buf_lhs
                    .iter()
                    .zip(buf_rhs.iter())
                    .map(|(x, y)| u8::$bitop_name(x, y))
                    .collect()
            }
        }
    };
}

bop!(ByteAnd, byte_and, bitand);
bop!(ByteOr, byte_or, bitor);
bop!(ByteXor, byte_xor, bitxor);

pub trait Hamming {
    fn hamming_weight(&self) -> u16;
    fn hamming_distance(_: &Bytes, _: &Bytes) -> u16;
}

impl Hamming for Bytes {
    fn hamming_weight(&self) -> u16 {
        let mut count = 0;
        let buf = self.into_buf();

        for b in buf.iter() {
            count += b.count_ones();
        }

        count as u16
    }

    fn hamming_distance(lhs: &Bytes, rhs: &Bytes) -> u16 {
        Bytes::byte_xor(lhs.clone(), rhs.clone()).hamming_weight()
    }
}

pub trait Foldable {
    fn fold(&self, size: usize) -> Result<Bytes, String>;
}

impl Foldable for Bytes {
    fn fold(&self, size: usize) -> Result<Bytes, String> {
        let n = self.len();
        if n % size == 0 {
            let k = self.len() / size;
            let mut result = self.slice(0, size);
            for i in 1..k {
                result = result.byte_xor(self.slice(i * size, (i + 1) * size))
            }
            Ok(result)
        } else {
            Err("Not a divisor".to_string())
        }
    }
}
