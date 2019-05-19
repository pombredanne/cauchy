use std::collections::HashSet;

use bytes::Bytes;
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use failure::Error;
use log::info;
use secp256k1::key::PublicKey;
use secp256k1::Signature;
use tokio::codec::{Decoder, Encoder};

use crate::{
    crypto::{
        signatures::ecdsa::*,
        sketches::{dummy_sketch::*, odd_sketch::*},
    },
    primitives::{transaction::*, varint::VarInt},
    utils::{constants::*, errors::MalformedMessageError, parsing::*},
};

macro_rules! encoding_info {
    ($($arg:tt)*) => {
        if config.debugging.decoding_verbose {
            info!(target: "encoding_event", $($arg)*);
        }
    };
}

macro_rules! decoding_info {
    ($($arg:tt)*) => {
        if config.debugging.decoding_verbose {
            info!(target: "decoding_event", $($arg)*);
        }
    };
}

pub enum Message {
    StartHandshake {
        secret: u64,
    }, // 0 || Secret VarInt
    EndHandshake {
        pubkey: PublicKey,
        sig: Signature,
    }, // 1 || Pk || Sig
    Nonce {
        nonce: u64,
    }, // 2 || nonce VarInt
    Work {
        oddsketch: OddSketch,
        root: Bytes,
        nonce: u64,
    }, // 3 || OddSketch || Root || Nonce
    MiniSketch {
        minisketch: DummySketch,
    }, // 4 || Number of Rows VarInt || IBLT
    GetTransactions {
        ids: HashSet<Bytes>,
    }, // 5 || Number of Ids VarInt || Ids
    Transactions {
        txs: Vec<Transaction>,
    }, // 6 || Number of Bytes VarInt || Tx ...
    Reconcile, // 7
    WorkAck,
    WorkNegAck,
    ReconcileNegAck,
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
                encoding_info!("encoding starthandshake");
                dst.put_u8(0);
                dst.extend(Bytes::from(VarInt::new(secret)));
            }
            Message::EndHandshake { pubkey, sig } => {
                encoding_info!("encoding endhandshake");
                dst.put_u8(1);
                dst.extend(bytes_from_pubkey(pubkey));
                dst.extend(bytes_from_sig(sig));
            }
            Message::Nonce { nonce } => {
                encoding_info!("encoding nonce");
                dst.put_u8(2);
                dst.extend(Bytes::from(VarInt::new(nonce)));
            }
            Message::Work {
                oddsketch,
                root,
                nonce,
            } => {
                encoding_info!("encoding work");
                dst.put_u8(3);
                // TODO: Variable length
                //dst.extend(Bytes::from(VarInt::new(sketch.len() as u64)));
                dst.extend(Bytes::from(oddsketch));
                dst.extend(root);
                dst.extend(Bytes::from(VarInt::new(nonce)));
            }
            Message::MiniSketch { minisketch } => {
                encoding_info!("encoding minisketch");
                dst.put_u8(4);
                dst.extend(Bytes::from(minisketch))
            }
            Message::GetTransactions { ids } => {
                encoding_info!("encoding tx request");
                dst.put_u8(5);
                dst.extend(Bytes::from(VarInt::new(ids.len() as u64)));
                for id in ids {
                    dst.extend(id);
                }
            }
            Message::Transactions { txs } => {
                encoding_info!("encoding txs");
                dst.put_u8(6);
                let mut payload = BytesMut::new();
                for tx in txs.into_iter() {
                    let raw = Bytes::from(tx);
                    payload.extend(raw);
                }

                let payload_len = payload.len() as u64;
                dst.extend(Bytes::from(VarInt::new(payload_len)));
                dst.extend(payload);
            }
            Message::Reconcile => dst.put_u8(7),
            Message::WorkAck => dst.put_u8(8),
            Message::WorkNegAck => dst.put_u8(9),
            Message::ReconcileNegAck => dst.put_u8(10),
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
                decoding_info!("decoding start handshake");
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
                decoding_info!("decoding end handshake");
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
                decoding_info!("decoding nonce");
                let (nonce_vi, len) = match VarInt::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };

                src.advance(1 + len);
                let msg = Message::Nonce {
                    nonce: u64::from(nonce_vi),
                };
                Ok(Some(msg))
            }
            3 => {
                decoding_info!("decoding work");
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

                let msg = Message::Work {
                    oddsketch: OddSketch::from(&sketch_dst[..]),
                    root: Bytes::from(&root_dst[..]),
                    nonce: u64::from(nonce_vi),
                };
                src.advance(1 + SKETCH_CAPACITY + HASH_LEN + len);
                Ok(Some(msg))
            }
            4 => {
                decoding_info!("decoding minisketch");
                let (minisketch, len) = match DummySketch::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };
                src.advance(1 + len);
                let msg = Message::MiniSketch { minisketch };
                Ok(Some(msg))
            }
            5 => {
                decoding_info!("decoding get transactions");
                let (n_tx_ids_vi, n_tx_ids_vi_len) = match VarInt::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };
                let us_n_tx_ids = usize::from(n_tx_ids_vi);
                decoding_info!("number of txns to decode {}", us_n_tx_ids);
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
            6 => {
                decoding_info!("decoding transactions");
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
                    decoding_info!("decoded transaction");
                }

                let msg = Message::Transactions { txs };
                src.advance(1 + payload_len_len + payload_len);
                Ok(Some(msg))
            }
            7 => {
                decoding_info!("decoding reconcile");
                src.advance(1);
                Ok(Some(Message::Reconcile))
            }
            8 => {
                decoding_info!("decoding work ack");
                src.advance(1);
                Ok(Some(Message::WorkAck))
            }
            9 => {
                decoding_info!("decoding work ack");
                src.advance(1);
                Ok(Some(Message::WorkNegAck))
            }
            10 => {
                decoding_info!("decoding reconcile negack");
                src.advance(1);
                Ok(Some(Message::ReconcileNegAck))
            }
            _ => {
                // TODO: Remove malformed msgs
                decoding_info!(
                    "received malformed msg: {}",
                    String::from_utf8_lossy(&src.clone().freeze())
                );
                Err(MalformedMessageError.into())
            }
        }
    }
}
