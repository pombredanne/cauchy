use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use crypto::signatures::ecdsa::*;
use crypto::sketches::dummy_sketch::*;
use primitives::transaction::Transaction;
use primitives::varint::VarInt;
use primitives::work_site::WorkSite;
use utils::constants::*;
use utils::parsing::*;

use failure::Error;

use utils::errors::{TransactionDeserialisationError, VarIntDeserialisationError};

pub trait TryFrom<T>: Sized {
    type Err;
    fn try_from(_: T) -> Result<Self, Self::Err>;
}

impl From<VarInt> for Bytes {
    fn from(varint: VarInt) -> Bytes {
        let mut n = u64::from(varint);
        let mut tmp = vec![];
        let mut len = 0;

        loop {
            tmp.put((0x7f & n) as u8 | (if len == 0 { 0x00 } else { 0x80 }));
            if n <= 0x7f {
                break;
            }
            n = (n >> 7) - 1;
            len += 1;
        }
        tmp.reverse();
        Bytes::from(tmp) // TODO: Replace with bufmut
    }
}

impl TryFrom<Bytes> for VarInt {
    type Err = Error;
    fn try_from(raw: Bytes) -> Result<VarInt, Self::Err> {
        let mut n: u64 = 0;
        let mut buf = raw.into_buf();
        loop {
            if buf.remaining() == 0 {
                return Err(VarIntDeserialisationError.into());
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

impl From<Transaction> for Bytes {
    fn from(tx: Transaction) -> Bytes {
        let mut buf = vec![];

        let vi_time = VarInt::from(*tx.get_time());
        buf.put(&Bytes::from(vi_time));

        let aux_data = tx.get_aux();
        let vi_aux_len = VarInt::from(aux_data.len());
        buf.put(&Bytes::from(vi_aux_len));
        buf.put(aux_data);

        let binary = tx.get_binary();
        let vi_binary_len = VarInt::from(binary.len());
        buf.put(&Bytes::from(vi_binary_len));
        buf.put(binary);

        Bytes::from(buf) // TODO: Replace with bufmut
    }
}

impl TryFrom<Bytes> for Transaction {
    type Err = Error;
    fn try_from(raw: Bytes) -> Result<Transaction, Self::Err> {
        let mut buf = raw.into_buf();

        let (vi_time, _) = match VarInt::parse_buf(&mut buf) {
            Ok(None) => return Err(TransactionDeserialisationError::TimeVarInt.into()),
            Err(err) => return Err(TransactionDeserialisationError::TimeVarInt.into()),
            Ok(Some(some)) => some,
        };

        let (vi_aux_len, _) = match VarInt::parse_buf(&mut buf) {
            Ok(None) => return Err(TransactionDeserialisationError::AuxVarInt.into()),
            Err(err) => return Err(TransactionDeserialisationError::AuxVarInt.into()),
            Ok(Some(some)) => some,
        };
        let us_aux_len = usize::from(vi_aux_len);
        if buf.remaining() < us_aux_len {
            return Err(TransactionDeserialisationError::AuxTooShort.into());
        }
        let mut dst_aux = vec![0; us_aux_len];
        buf.copy_to_slice(&mut dst_aux);

        let (vi_bin_len, _) = match VarInt::parse_buf(&mut buf) {
            Ok(None) => return Err(TransactionDeserialisationError::BinaryVarInt.into()),
            Err(err) => return Err(TransactionDeserialisationError::BinaryVarInt.into()),
            Ok(Some(some)) => some,
        };
        let us_bin_len = usize::from(vi_bin_len);
        if buf.remaining() < us_bin_len {
            return Err(TransactionDeserialisationError::BinaryTooShort.into());
        }
        let mut dst_bin = vec![0; us_bin_len];
        buf.copy_to_slice(&mut dst_bin);

        Ok(Transaction::new(
            u64::from(vi_time),
            Bytes::from(dst_aux),
            Bytes::from(dst_bin),
        ))
    }
}

impl From<WorkSite> for Bytes {
    fn from(work_site: WorkSite) -> Bytes {
        let mut buf = BytesMut::with_capacity(PUBKEY_LEN + 32 + 8);
        let pk = bytes_from_pubkey(work_site.get_public_key());
        buf.extend_from_slice(&pk[..]);
        buf.extend_from_slice(&work_site.get_root());
        buf.put_u64_be(work_site.get_nonce());
        buf.freeze()
    }
}

impl From<DummySketch> for Bytes {
    fn from(dummysketch: DummySketch) -> Bytes {
        let pos_len = dummysketch.pos_len();
        let vi_pos_len = VarInt::from(pos_len);
        let mut buf = BytesMut::with_capacity(pos_len + vi_pos_len.len());

        buf.extend(&Bytes::from(vi_pos_len));
        for item in dummysketch.get_pos() {
            buf.extend(item)
        }
        buf.freeze()
    }
}
