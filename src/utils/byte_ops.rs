extern crate bytes;
use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use std::ops::*;

macro_rules! from_bytes {
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
                let mut result = BytesMut::with_capacity(self.len());
                let buf_lhs = self.into_buf();
                let buf_rhs = rhs.into_buf();

                for (x, y) in buf_lhs.iter().zip(buf_rhs.iter()) {
                    result.put(u8::$bitop_name(x, y));
                }
                Bytes::from(result)
            }
        }
    };
}

bop!(ByteAnd, byte_and, bitand);
bop!(ByteOr, byte_or, bitor);
bop!(ByteXor, byte_xor, bitxor);

pub trait Hamming {
    fn hamming_weight(&self) -> u16;
    fn hamming_distance(&Bytes, &Bytes) -> u16;
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
    fn fold(&self, usize) -> Result<Bytes, String>;
}

impl Foldable for Bytes {
    fn fold(&self, m: usize) -> Result<Bytes, String> {
        let n = self.len();
        if n % m == 0 {
            let k = self.len() / m;
            let mut result = self.slice(0, m);
            for i in 1..k {
                result = result.byte_xor(self.slice(i * m, (i + 1) * m))
            }
            Ok(result)
        } else {
            Err("Not a divisor".to_string())
        }
    }
}
