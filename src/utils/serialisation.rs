use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use crypto::signatures::ecdsa::*;
use primitives::script::Script;
use primitives::work_site::WorkSite;
use primitives::{transaction::Transaction, transaction_state::TransactionState, varint::VarInt};
use std::collections::HashSet;
use utils::constants::*;

pub trait TryFrom<T>: Sized {
    type Err;
    fn try_from(_: T) -> Result<Self, Self::Err>;
} // Isn't this stable now??

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
        Bytes::from(tmp)
    }
}

impl TryFrom<Bytes> for VarInt {
    type Err = String;
    fn try_from(raw: Bytes) -> Result<VarInt, Self::Err> {
        let mut n: u64 = 0;
        let mut buf = raw.into_buf();
        loop {
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

        let var_time = VarInt::from(tx.time());
        buf.put(&Bytes::from(var_time));

        let n_spendable = VarInt::from(tx.n_spendable());
        buf.put(&Bytes::from(n_spendable));

        for script in Vec::from(tx) {
            let script_raw = Bytes::from(script);
            buf.put(&Bytes::from(VarInt::from(script_raw.len())));
            buf.put(&script_raw);
        }
        Bytes::from(buf)
    }
}

// TODO: Catch errors
impl TryFrom<Bytes> for Transaction {
    type Err = String;
    fn try_from(raw: Bytes) -> Result<Transaction, Self::Err> {
        let mut scripts = Vec::new();
        let mut buf = raw.into_buf();

        let vi_time = VarInt::parse_buf(&mut buf);
        let n_spendable = VarInt::parse_buf(&mut buf);

        while buf.has_remaining() {
            let vi = VarInt::parse_buf(&mut buf);

            let len = usize::from(vi);
            let mut dst = vec![0; len as usize];
            buf.copy_to_slice(&mut dst);

            scripts.push(Script::new(Bytes::from(dst)));
        }
        Ok(Transaction::new(
            u64::from(vi_time),
            u32::from(n_spendable),
            scripts,
        ))
    }
}

impl From<Bytes> for TransactionState {
    fn from(raw: Bytes) -> TransactionState {
        let mut buf = raw.into_buf();

        let mut hash_set: HashSet<u32> = HashSet::new();
        while buf.has_remaining() {
            hash_set.insert(buf.get_u32_be());
        }

        TransactionState::new(hash_set)
    }
}

impl From<TransactionState> for Bytes {
    fn from(tx_state: TransactionState) -> Bytes {
        let mut buf = vec![];
        for val in tx_state.iter() {
            buf.put_u32_be(*val);
        }
        Bytes::from(buf)
    }
}

impl From<WorkSite> for Bytes {
    fn from(work_site: WorkSite) -> Bytes {
        let mut bytes = BytesMut::with_capacity(PUBKEY_LEN + 8);
        let pk = bytes_from_pubkey(work_site.get_public_key());
        bytes.extend_from_slice(&pk[..]);
        bytes.put_u64_be(work_site.get_nonce());
        bytes.freeze()
    }
}
