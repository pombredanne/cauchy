use bytes::{Buf, Bytes};
use primitives::transaction::*;
use primitives::varint::*;

pub trait Parsable<U> {
    fn parse_buf<T: Buf>(buf: &mut T) -> Result<U, String>;
}

impl Parsable<Transaction> for Transaction {
    fn parse_buf<T: Buf>(buf: &mut T) -> Result<Transaction, String> {
        // TODO: Catch errors
        let vi_time = VarInt::parse_buf(buf)?;

        let vi_aux_len = VarInt::parse_buf(buf)?;
        let mut dst_aux = vec![0; usize::from(vi_aux_len)];
        buf.copy_to_slice(&mut dst_aux);

        let vi_bin_len = VarInt::parse_buf(buf)?;
        let mut dst_bin = vec![0; usize::from(vi_bin_len)];
        buf.copy_to_slice(&mut dst_bin);

        Ok(Transaction::new(
            u64::from(vi_time),
            Bytes::from(dst_aux),
            Bytes::from(dst_bin),
        ))
    }
}

impl Parsable<VarInt> for VarInt {
    fn parse_buf<T: Buf>(buf: &mut T) -> Result<VarInt, String> {
        let mut n: u64 = 0;
        loop {
            if buf.remaining() == 0 {
                return Err("No remaining bytes".to_string());
            }
            let k = buf.get_u8();
            n = (n << 7) | u64::from(k & 0x7f);
            if 0x00 != (k & 0x80) {
                n += 1;
            } else {
                return Ok(VarInt::new(n));
            }
        }
    }
}
