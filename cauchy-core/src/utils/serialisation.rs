use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use crypto::signatures::ecdsa::*;
use crypto::sketches::iblt::*;
use primitives::transaction::Transaction;
use primitives::varint::VarInt;
use primitives::work_site::WorkSite;
use state::spend_state::*;
use std::collections::HashSet;
use utils::constants::*;
use utils::parsing::*;

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
    type Err = String;
    fn try_from(raw: Bytes) -> Result<VarInt, Self::Err> {
        let mut n: u64 = 0;
        let mut buf = raw.into_buf();
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

// TODO: Catch errors
impl TryFrom<Bytes> for Transaction {
    type Err = String;
    fn try_from(raw: Bytes) -> Result<Transaction, Self::Err> {
        let mut buf = raw.into_buf();

        let vi_time = VarInt::parse_buf(&mut buf)?;

        let vi_aux_len = VarInt::parse_buf(&mut buf)?;
        let mut dst_aux = vec![0; usize::from(vi_aux_len)];
        buf.copy_to_slice(&mut dst_aux);

        let vi_bin_len = VarInt::parse_buf(&mut buf)?;
        let mut dst_bin = vec![0; usize::from(vi_bin_len)];
        buf.copy_to_slice(&mut dst_bin);

        Ok(Transaction::new(
            u64::from(vi_time),
            Bytes::from(dst_aux),
            Bytes::from(dst_bin),
        ))
    }
}

impl From<Bytes> for SpendState {
    fn from(raw: Bytes) -> SpendState {
        let mut buf = raw.into_buf();

        let mut hash_set: HashSet<u32> = HashSet::new();
        while buf.has_remaining() {
            hash_set.insert(buf.get_u32_be());
        }

        SpendState::new(hash_set)
    }
}

impl From<SpendState> for Bytes {
    fn from(tx_state: SpendState) -> Bytes {
        let mut buf = vec![];
        for val in tx_state.iter() {
            buf.put_u32_be(*val);
        }
        Bytes::from(buf) // TODO: Replace with bufmut
    }
}

impl From<WorkSite> for Bytes {
    fn from(work_site: WorkSite) -> Bytes {
        let mut buf = BytesMut::with_capacity(PUBKEY_LEN + 8);
        let pk = bytes_from_pubkey(work_site.get_public_key());
        buf.extend_from_slice(&pk[..]);
        buf.put_u64_be(work_site.get_nonce());
        buf.freeze()
    }
}

impl From<IBLT> for Bytes {
    fn from(iblt: IBLT) -> Bytes {
        let n_rows = iblt.get_rows().len();
        let total_size = n_rows * (IBLT_CHECKSUM_LEN + IBLT_PAYLOAD_LEN + 8) + 8;
        let mut buf = BytesMut::with_capacity(total_size);
        buf.put_u32_be(n_rows as u32);
        for row in iblt.get_rows() {
            buf.put_i32_be(row.get_count());
            buf.extend_from_slice(&row.get_payload()[..]);
            buf.extend_from_slice(&row.get_checksum()[..]);
        }
        buf.freeze()
    }
}

impl From<Bytes> for IBLT {
    fn from(raw: Bytes) -> IBLT {
        let mut buf = raw.into_buf();
        let n_rows = buf.get_u32_be() as usize;
        let mut rows = Vec::with_capacity(n_rows);
        for _ in 0..n_rows {
            let count = buf.get_i32_be();
            let mut dst_payload = vec![0; IBLT_PAYLOAD_LEN];
            buf.copy_to_slice(&mut dst_payload);
            let mut dst_checksum = vec![0; IBLT_CHECKSUM_LEN];
            buf.copy_to_slice(&mut dst_checksum);
            rows.push(Row::new(
                count,
                Bytes::from(&dst_payload[..]),
                Bytes::from(&dst_checksum[..]),
            ));
        }

        IBLT::from_rows(rows, IBLT_N_HASHES)
    }
}

impl From<Row> for Bytes {
    fn from(row: Row) -> Bytes {
        let mut buf = BytesMut::with_capacity(4 + IBLT_CHECKSUM_LEN + IBLT_PAYLOAD_LEN);
        buf.put_i32_be(row.get_count());
        buf.put(&row.get_payload()[..]);
        buf.put(&row.get_checksum()[..]);
        Bytes::from(buf)
    }
}

impl From<Bytes> for Row {
    fn from(raw: Bytes) -> Row {
        let mut buf = raw.into_buf();
        let count = buf.get_i32_be();
        let mut dst_payload = vec![0; IBLT_PAYLOAD_LEN];
        buf.copy_to_slice(&mut dst_payload);
        let mut dst_checksum = vec![0; IBLT_CHECKSUM_LEN];
        buf.copy_to_slice(&mut dst_checksum);
        Row::new(
            count,
            Bytes::from(&dst_payload[..]),
            Bytes::from(&dst_checksum[..]),
        )
    }
}
