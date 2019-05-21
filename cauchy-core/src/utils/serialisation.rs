use std::{convert::TryFrom, net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr}};

use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use failure::Error;

use crate::{
    crypto::{signatures::ecdsa::*, sketches::dummy_sketch::*},
    primitives::{
        access_pattern::*, transaction::Transaction, varint::VarInt, work_site::WorkSite,
    },
    net::peers::{Peers, Peer}
};

use super::{
    constants::*,
    errors::{TransactionDeserialisationError, VarIntDeserialisationError, PeerDeserialisationError},
    parsing::*,
};

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
    type Error = Error;
    fn try_from(raw: Bytes) -> Result<VarInt, Self::Error> {
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

        let vi_time = VarInt::from(tx.get_time());
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
    type Error = Error;
    fn try_from(raw: Bytes) -> Result<Transaction, Self::Error> {
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

impl From<AccessPattern> for Bytes {
    fn from(access_pattern: AccessPattern) -> Bytes {
        let vi_read_len = VarInt::new(access_pattern.read.len() as u64);
        let mut raw = BytesMut::with_capacity(
            vi_read_len.len()
                + access_pattern.read.len() * HASH_LEN
                + access_pattern.write.len() * (HASH_LEN + VALUE_LEN),
        );

        // Put num of reads
        raw.put(Bytes::from(vi_read_len));

        // Put read keys
        for key in access_pattern.read.iter() {
            raw.put(key);
        }

        // Put writes
        for (key, value) in access_pattern.write.iter() {
            raw.put(key);
            raw.put(value);
        }
        raw.freeze()
    }
}

impl From<Peer> for Bytes {
    fn from(peer: Peer) -> Bytes {
        let addr = peer.get_addr();
        let ip = match addr.ip() {
            IpAddr::V4(some) => some.octets(),
            _ => unreachable!() // TODO: IPV6
        };
        let port = addr.port();
        let mut raw = BytesMut::with_capacity(6);
        raw.put(&ip[..]);
        raw.put_u16_be(port);
        raw.freeze()
    }
}

impl From<Bytes> for Peer {
    fn from(raw: Bytes) -> Peer {
        let mut buf = raw.into_buf();

        let mut dst_ip = [0; 4];
        buf.copy_to_slice(&mut dst_ip);

        let port = buf.get_u16_be();

        Peer::new(SocketAddr::from((dst_ip, port)))
    }
}

impl TryFrom<Bytes> for Peers {
    type Error = Error;
    fn try_from(raw: Bytes) -> Result<Peers, Error> {
        let mut buf = raw.into_buf();
        let (vi_n, _) = match VarInt::parse_buf(&mut buf) {
            Ok(None) => return Err(PeerDeserialisationError.into()),
            Err(err) => return Err(PeerDeserialisationError.into()),
            Ok(Some(some)) => some,
        };
        
        let n = usize::from(vi_n);
        if buf.remaining() < 6 * n {
            return Err(PeerDeserialisationError.into())
        }

        let mut vec_peers = vec![];
        for i in 0..n {
            let mut dst = vec![0; 6];
            buf.copy_to_slice(&mut dst);
            vec_peers.push(Peer::from(Bytes::from(&dst[..])));
        }

        Ok(Peers::new(vec_peers))


    }
}

impl From<Peers> for Bytes {
    fn from(peers: Peers) -> Bytes {
        let vec_peers = peers.to_vec();
        let usize_n = vec_peers.len();
        let mut buf = BytesMut::with_capacity(6 * usize_n);

        let vi_n = VarInt::from(usize_n as u64);
        buf.put(&Bytes::from(vi_n));

        for peer in vec_peers {
            buf.put(Bytes::from(peer));
        }

        buf.freeze()
    }
}