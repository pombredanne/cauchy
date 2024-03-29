use std::collections::HashSet;

use bytes::Bytes;
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use failure::Error;
use log::info;
use secp256k1::key::PublicKey;
use secp256k1::Signature;
use tokio::codec::{Decoder, Encoder};

use super::peers::{Peer, Peers};
use crate::{
    crypto::{
        signatures::ecdsa::*,
        sketches::{dummy_sketch::*, odd_sketch::*},
    },
    primitives::{
        transaction::*,
        varint::VarInt,
        work::{WorkStack, WorkState},
    },
    utils::{constants::*, errors::MalformedMessageError, parsing::*},
};

pub enum Message {
    StartHandshake { secret: u64 }, // 0 || Secret VarInt
    EndHandshake { pubkey: PublicKey, sig: Signature }, // 1 || Pk || Sig
    Work(WorkStack),                // 2 || OddSketch || Root || Nonce
    MiniSketch { minisketch: DummySketch }, // 3 || Number of Rows VarInt || IBLT
    GetTransactions { ids: HashSet<Bytes> }, // 4 || Number of Ids VarInt || Ids
    Transactions { txs: Vec<Transaction> }, // 5 || Number of Bytes VarInt || Tx ...
    Reconcile,                      // 6
    ReconcileNegAck,                // 7
    GetWork,                        // 8
    Peers { peers: Peers },         // 9 || Number of peers || Peers
}

pub struct MessageCodec;

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // TODO: Manage capacity better
        dst.reserve(1);
        match item {
            Message::StartHandshake { secret } => {
                info!(target: "encoding_event", "encoding starthandshake");
                dst.put_u8(0);
                dst.extend(Bytes::from(VarInt::new(secret)));
            }
            Message::EndHandshake { pubkey, sig } => {
                info!(target: "encoding_event", "encoding endhandshake");
                dst.put_u8(1);
                dst.extend(bytes_from_pubkey(pubkey));
                dst.extend(bytes_from_sig(sig));
            }
            Message::Work(work_stack) => {
                info!(target: "encoding_event", "encoding work");
                dst.put_u8(2);
                // TODO: Variable length
                //dst.extend(Bytes::from(VarInt::new(sketch.len() as u64)));
                dst.extend(Bytes::from(work_stack.get_oddsketch()));
                dst.extend(work_stack.get_root());
                dst.extend(Bytes::from(VarInt::new(work_stack.get_nonce())));
            }
            Message::MiniSketch { minisketch } => {
                info!(target: "encoding_event", "encoding minisketch");
                dst.put_u8(3);
                dst.extend(Bytes::from(minisketch))
            }
            Message::GetTransactions { ids } => {
                info!(target: "encoding_event", "encoding tx request");
                dst.put_u8(4);
                dst.extend(Bytes::from(VarInt::new(ids.len() as u64)));
                for id in ids {
                    dst.extend(id);
                }
            }
            Message::Transactions { txs } => {
                info!(target: "encoding_event", "encoding txs");
                dst.put_u8(5);
                let mut payload = BytesMut::new();
                for tx in txs.into_iter() {
                    let raw = Bytes::from(tx);
                    payload.extend(raw);
                }

                let payload_len = payload.len() as u64;
                dst.extend(Bytes::from(VarInt::new(payload_len)));
                dst.extend(payload);
            }
            Message::Reconcile => dst.put_u8(6),
            Message::ReconcileNegAck => dst.put_u8(7),
            Message::GetWork => dst.put_u8(8),
            Message::Peers { peers } => {
                dst.put_u8(9);
                dst.extend(Bytes::from(peers));
            }
        }
        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut buf = src.clone().into_buf();

        if buf.remaining() == 0 {
            return Ok(None);
        }

        match buf.get_u8() {
            0 => {
                info!(target: "decoding_event", "decoding start handshake");
                let (preimage_vi, len) = match VarInt::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };

                src.advance(1 + len);
                let msg = Message::StartHandshake {
                    secret: u64::from(preimage_vi),
                };
                Ok(Some(msg))
            }
            1 => {
                info!(target: "decoding_event", "decoding end handshake");
                if buf.remaining() < PUBKEY_LEN + SIG_LEN {
                    return Ok(None);
                }
                let mut pubkey_dst = [0; PUBKEY_LEN];
                buf.copy_to_slice(&mut pubkey_dst);
                let pubkey = pubkey_from_bytes(Bytes::from(&pubkey_dst[..]))?;

                let mut sig_dst = [0; SIG_LEN];
                buf.copy_to_slice(&mut sig_dst);
                let sig = sig_from_bytes(Bytes::from(&sig_dst[..]))?;
                src.advance(1 + PUBKEY_LEN + SIG_LEN);
                let msg = Message::EndHandshake { pubkey, sig };
                Ok(Some(msg))
            }
            2 => {
                info!(target: "decoding_event", "decoding work");
                if buf.remaining() < SKETCH_CAPACITY + HASH_LEN {
                    return Ok(None);
                }
                let mut sketch_dst = [0; SKETCH_CAPACITY];
                buf.copy_to_slice(&mut sketch_dst);

                let mut root_dst = [0; HASH_LEN];
                buf.copy_to_slice(&mut root_dst);

                let (nonce_vi, len) = match VarInt::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };

                let msg = Message::Work(WorkStack::new(
                    Bytes::from(&root_dst[..]),
                    OddSketch::from(&sketch_dst[..]),
                    u64::from(nonce_vi),
                ));
                src.advance(1 + SKETCH_CAPACITY + HASH_LEN + len);
                Ok(Some(msg))
            }
            3 => {
                info!(target: "decoding_event", "decoding minisketch");
                let (minisketch, len) = match DummySketch::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };
                src.advance(1 + len);
                let msg = Message::MiniSketch { minisketch };
                Ok(Some(msg))
            }
            4 => {
                info!(target: "decoding_event", "decoding get transactions");
                let (n_tx_ids_vi, n_tx_ids_vi_len) = match VarInt::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };
                let us_n_tx_ids = usize::from(n_tx_ids_vi);
                info!(target: "decoding_event", "number of txns to decode {}", us_n_tx_ids);
                let total_size = us_n_tx_ids * HASH_LEN;
                let mut ids = HashSet::with_capacity(us_n_tx_ids);

                if buf.remaining() < total_size {
                    Ok(None)
                } else {
                    for _ in 0..us_n_tx_ids {
                        let mut id_dst = [0; HASH_LEN];
                        buf.copy_to_slice(&mut id_dst);
                        ids.insert(Bytes::from(&id_dst[..]));
                    }
                    let msg = Message::GetTransactions { ids };
                    src.advance(1 + n_tx_ids_vi_len + total_size);
                    Ok(Some(msg))
                }
            }
            5 => {
                info!(target: "decoding_event", "decoding transactions");
                let (payload_len_vi, payload_len_len) = match VarInt::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };
                let payload_len = usize::from(payload_len_vi);

                if buf.remaining() < payload_len {
                    return Ok(None);
                }

                let mut txs = vec![]; // TODO: Estimate of size here?

                while buf.remaining() > 0 {
                    let (tx, _) = match Transaction::parse_buf(&mut buf)? {
                        Some(some) => some,
                        None => return Ok(None),
                    };
                    txs.push(tx);
                    info!(target: "decoding_event", "decoded transaction");
                }

                let msg = Message::Transactions { txs };
                src.advance(1 + payload_len_len + payload_len);
                Ok(Some(msg))
            }
            6 => {
                info!(target: "decoding_event", "decoding reconcile");
                src.advance(1);
                Ok(Some(Message::Reconcile))
            }
            7 => {
                info!(target: "decoding_event", "decoding reconcile negack");
                src.advance(1);
                Ok(Some(Message::ReconcileNegAck))
            }
            8 => {
                info!(target: "decoding_event", "decoding get work");
                src.advance(1);
                Ok(Some(Message::GetWork))
            }
            90 => {
                info!(target: "decoding_event", "decoding peers");
                let (vi_n, _) = match VarInt::parse_buf(&mut buf)? {
                    None => return Ok(None),
                    Some(some) => some,
                };

                let n = usize::from(vi_n);
                if buf.remaining() < 6 * n {
                    return Ok(None);
                }

                let mut vec_peers = vec![];
                for i in 0..n {
                    let mut dst = vec![0; 6];
                    buf.copy_to_slice(&mut dst);
                    vec_peers.push(Peer::from(Bytes::from(&dst[..])));
                }

                Ok(Some(Message::Peers {
                    peers: Peers::new(vec_peers),
                }))
            }
            _ => {
                // TODO: Remove malformed msgs
                info!(target: "decoding_event",
                    "received malformed msg: {}",
                    String::from_utf8_lossy(&src.clone().freeze())
                );
                Err(MalformedMessageError.into())
            }
        }
    }
}
