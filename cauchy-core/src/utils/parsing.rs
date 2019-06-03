use bytes::{Buf, Bytes};
use failure::Error;
use log::info;

use std::collections::HashSet;

use crate::{
    crypto::sketches::{dummy_sketch::*, *},
    primitives::{transaction::*, varint::*},
};

use super::{constants::*, errors::VarIntParseError};

pub trait Parsable<U> {
    fn parse_buf<T: Buf>(buf: &mut T) -> Result<Option<(U, usize)>, Error>;
}

impl Parsable<Transaction> for Transaction {
    fn parse_buf<T: Buf>(buf: &mut T) -> Result<Option<(Transaction, usize)>, Error> {
        info!(target: "parsing_event", "begin tx parsing");
        let (vi_time, vi_time_len) = match VarInt::parse_buf(buf)? {
            Some(some) => some,
            None => return Ok(None),
        };
        let (vi_aux_len, vi_aux_len_len) = match VarInt::parse_buf(buf)? {
            Some(some) => some,
            None => return Ok(None),
        };
        let us_aux_len = usize::from(vi_aux_len);
        if buf.remaining() < us_aux_len {
            return Ok(None);
        }
        let mut dst_aux = vec![0; us_aux_len];
        buf.copy_to_slice(&mut dst_aux);
        let (vi_bin_len, vi_bin_len_len) = match VarInt::parse_buf(buf)? {
            Some(some) => some,
            None => return Ok(None),
        };
        let us_bin_len = usize::from(vi_bin_len);
        if buf.remaining() < us_bin_len {
            return Ok(None);
        }
        let mut dst_bin = vec![0; us_bin_len];
        buf.copy_to_slice(&mut dst_bin);
        info!(target: "parsing_event", "finished tx parsing");
        Ok(Some((
            Transaction::new(
                u64::from(vi_time),
                Bytes::from(dst_aux),
                Bytes::from(dst_bin),
            ),
            vi_time_len + vi_aux_len_len + us_aux_len + vi_bin_len_len + us_bin_len,
        )))
    }
}

impl Parsable<VarInt> for VarInt {
    fn parse_buf<T: Buf>(buf: &mut T) -> Result<Option<(VarInt, usize)>, Error> {
        info!(target: "parsing_event", "begin varint parsing");
        let mut n: u64 = 0;
        let mut len = 0;
        loop {
            if buf.remaining() == 0 {
                if len < 8 {
                    return Ok(None);
                } else {
                    return Err(VarIntParseError { len }.into());
                }
            }
            let k = buf.get_u8();
            len += 1;
            n = (n << 7) | u64::from(k & 0x7f);
            if 0x00 != (k & 0x80) {
                n += 1;
            } else {
                info!(target: "parsing_event", "finished varint parsing");
                return Ok(Some((VarInt::new(n), len)));
            }
        }
    }
}

impl Parsable<DummySketch> for DummySketch {
    fn parse_buf<T: Buf>(buf: &mut T) -> Result<Option<(DummySketch, usize)>, Error> {
        // TODO: Catch errors
        info!(target: "parsing_event", "begin minisketch parsing");
        let (vi_pos_len, vi_pos_len_len) = match VarInt::parse_buf(buf)? {
            Some(some) => some,
            None => return Ok(None),
        };
        let us_pos_len = usize::from(vi_pos_len);
        let mut pos_set = HashSet::with_capacity(us_pos_len);
        for i in 0..us_pos_len {
            info!(target: "parsing_event", "ID {} of {}", i, us_pos_len);
            if buf.remaining() < HASH_LEN {
                return Ok(None);
            }
            let mut dst_aux = vec![0; HASH_LEN];
            buf.copy_to_slice(&mut dst_aux);
            pos_set.insert(Bytes::from(dst_aux));
        }
        info!(target: "parsing_event", "finished minisketch parsing");
        Ok(Some((
            DummySketch::sketch_ids(&pos_set),
            vi_pos_len_len + us_pos_len * HASH_LEN,
        )))
    }
}
