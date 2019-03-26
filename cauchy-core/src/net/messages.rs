use std::collections::HashSet;

use bytes::Bytes;
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use failure::Error;
use secp256k1::key::PublicKey;
use secp256k1::Signature;
use tokio::codec::{Decoder, Encoder};

use crypto::signatures::ecdsa::*;
use crypto::sketches::dummy_sketch::*;
use crypto::sketches::odd_sketch::*;
use primitives::transaction::*;
use primitives::varint::VarInt;
use utils::constants::*;
use utils::errors::MalformedMessageError;
use utils::parsing::*;

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
        txs: HashSet<Transaction>,
    }, // 6 || Number of Bytes VarInt || Tx ...
    Reconcile, // 7
    WorkAck
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
                if CONFIG.DEBUGGING.ENCODING_VERBOSE {
                    println!("encoding starthandshake");
                }
                dst.put_u8(0);
                dst.extend(Bytes::from(VarInt::new(secret)));
            }
            Message::EndHandshake { pubkey, sig } => {
                if CONFIG.DEBUGGING.ENCODING_VERBOSE {
                    println!("encoding endhandshake");
                }
                dst.put_u8(1);
                dst.extend(bytes_from_pubkey(pubkey));
                dst.extend(bytes_from_sig(sig));
            }
            Message::Nonce { nonce } => {
                if CONFIG.DEBUGGING.ENCODING_VERBOSE {
                    println!("encoding nonce");
                }
                dst.put_u8(2);
                dst.extend(Bytes::from(VarInt::new(nonce)));
            }
            Message::Work {
                oddsketch,
                root,
                nonce,
            } => {
                if CONFIG.DEBUGGING.ENCODING_VERBOSE {
                    println!("encoding work");
                }
                dst.put_u8(3);
                // TODO: Variable length
                //dst.extend(Bytes::from(VarInt::new(sketch.len() as u64)));
                dst.extend(Bytes::from(oddsketch));
                dst.extend(root);
                dst.extend(Bytes::from(VarInt::new(nonce)));
            }
            Message::MiniSketch { minisketch } => {
                if CONFIG.DEBUGGING.ENCODING_VERBOSE {
                    println!("encoding minisketch");
                }
                dst.put_u8(4);
                dst.extend(Bytes::from(minisketch))
            }
            Message::GetTransactions { ids } => {
                if CONFIG.DEBUGGING.ENCODING_VERBOSE {
                    println!("encoding tx request");
                }
                dst.put_u8(5);
                dst.extend(Bytes::from(VarInt::new(ids.len() as u64)));
                for id in ids {
                    dst.extend(id);
                }
            }
            Message::Transactions { txs } => {
                if CONFIG.DEBUGGING.ENCODING_VERBOSE {
                    println!("encoding txs");
                }
                dst.put_u8(6);
                let mut payload = BytesMut::new();
                let n_txs = txs.len() as u64;
                for tx in txs.into_iter() {
                    let raw = Bytes::from(tx);
                    payload.extend(raw);
                }

                dst.extend(Bytes::from(VarInt::new(n_txs)));
                dst.extend(payload);
            }
            Message::Reconcile => dst.put_u8(7),
            Message::WorkAck => dst.put_u8(8),
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
                if CONFIG.DEBUGGING.DECODING_VERBOSE {
                    println!("decoding nonce");
                }
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
                if CONFIG.DEBUGGING.DECODING_VERBOSE {
                    println!("decoding work");
                }
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
                if CONFIG.DEBUGGING.DECODING_VERBOSE {
                    println!("decoding minisketch");
                }
                let (minisketch, len) = match DummySketch::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };
                src.advance(1 + len);
                let msg = Message::MiniSketch { minisketch };
                Ok(Some(msg))
            }
            5 => {
                if CONFIG.DEBUGGING.DECODING_VERBOSE {
                    println!("decoding transaction request");
                }
                let (n_tx_ids_vi, n_tx_ids_vi_len) = match VarInt::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };
                let us_n_tx_ids = usize::from(n_tx_ids_vi);
                if CONFIG.DEBUGGING.DECODING_VERBOSE {
                    println!("number of txns to decode {}", us_n_tx_ids);
                }
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
                if CONFIG.DEBUGGING.DECODING_VERBOSE {
                    println!("decoding transactions");
                }
                let (n_tx_vi, n_tx_vi_len) = match VarInt::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };
                let n_tx = usize::from(n_tx_vi);
                if CONFIG.DEBUGGING.DECODING_VERBOSE {
                    println!("number of transactions {}", n_tx);
                }
                let mut total_size: usize = 0;
                let mut txs = HashSet::with_capacity(n_tx);
                for _i in 0..n_tx {
                    let (tx, tx_len) = match Transaction::parse_buf(&mut buf)? {
                        Some(some) => some,
                        None => return Ok(None),
                    };
                    txs.insert(tx);
                    if CONFIG.DEBUGGING.DECODING_VERBOSE {
                        println!("decoded transaction");
                    }
                    total_size += tx_len;
                }
                let msg = Message::Transactions { txs };
                src.advance(1 + n_tx_vi_len + total_size);
                Ok(Some(msg))
            }
            7 => {
                if CONFIG.DEBUGGING.DECODING_VERBOSE {
                    println!("decoding reconcile");
                }
                src.advance(1);
                Ok(Some(Message::Reconcile))
            },
            8 => {
                if CONFIG.DEBUGGING.DECODING_VERBOSE {
                    println!("decoding work ack");
                }
                src.advance(1);
                Ok(Some(Message::WorkAck))
            }
            _ => {
                // TODO: Remove malformed msgs
                println!(
                    "received malformed msg: {}",
                    String::from_utf8_lossy(&src.clone().freeze())
                );
                Err(MalformedMessageError.into())
            }
        }
    }
}
