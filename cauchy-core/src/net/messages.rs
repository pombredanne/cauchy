use bytes::Bytes;
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use std::collections::HashSet;
use tokio::codec::{Decoder, Encoder};
use tokio::io::{Error, ErrorKind};

use secp256k1::key::PublicKey;
use secp256k1::Signature;

use crypto::signatures::ecdsa::*;
use crypto::sketches::dummy_sketch::*;
use primitives::transaction::*;
use primitives::varint::VarInt;
use utils::constants::*;
use utils::parsing::*;

pub enum Message {
    StartHandshake { secret: u64 }, // 0 || Secret VarInt
    EndHandshake { pubkey: PublicKey, sig: Signature }, // 1 || Pk || Sig
    Nonce { nonce: u64 },           // 2 || nonce VarInt
    OddSketch { sketch: Bytes },    // 3 || Sketch
    MiniSketch { mini_sketch: DummySketch }, // 4 || Number of Rows VarInt || IBLT
    GetTransactions { ids: HashSet<Bytes> }, // 5 || Number of Ids VarInt || Ids
    Transactions { txs: Vec<Transaction> }, // 6 || Number of Bytes VarInt || Tx ...
    Reconcile,                      // 7
}

pub struct MessageCodec;

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // TODO: Manage capacity better
        match item {
            Message::StartHandshake { secret } => {
                dst.put_u8(0);
                dst.extend(Bytes::from(VarInt::new(secret)));
            }
            Message::EndHandshake { pubkey, sig } => {
                dst.put_u8(1);
                dst.extend(bytes_from_pubkey(pubkey));
                dst.extend(bytes_from_sig(sig));
            }
            Message::Nonce { nonce } => {
                dst.put_u8(2);
                dst.extend(Bytes::from(VarInt::new(nonce)));
            }
            Message::OddSketch { sketch } => {
                dst.put_u8(3);
                //dst.extend(Bytes::from(VarInt::new(sketch.len() as u64)));
                dst.extend(sketch);
            }
            Message::MiniSketch { mini_sketch } => {
                dst.put_u8(4);
                dst.extend(Bytes::from(mini_sketch))
            }
            Message::GetTransactions { ids } => {
                println!("Encoding get txns");
                dst.put_u8(5);
                dst.extend(Bytes::from(VarInt::new(ids.len() as u64)));
                println!("Number of txns to encode {}", ids.len());
                for id in ids {
                    dst.extend(id);
                }
            }
            Message::Transactions { txs } => {
                dst.put_u8(6);
                let mut payload = BytesMut::new();
                for tx in txs {
                    let raw = Bytes::from(tx);
                    payload.extend(Bytes::from(VarInt::new(raw.len() as u64)));
                    payload.extend(raw);
                }

                dst.extend(Bytes::from(VarInt::new(payload.len() as u64)));
                dst.extend(payload);
            }
            Message::Reconcile => dst.put_u8(7),
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
                let (preimage_vi, len) = match VarInt::parse_buf(&mut buf) {
                    Ok(Some(some)) => some,
                    Ok(None) => return Ok(None),
                    Err(err) => return Err(Error::new(ErrorKind::Other, err)),
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
                let pubkey = match pubkey_from_bytes(Bytes::from(&pubkey_dst[..])) {
                    Ok(some) => some,
                    Err(_) => {
                        // TODO: Remove malformed msgs
                        return Err(Error::new(ErrorKind::InvalidData, "Invalid Pubkey"));
                    }
                };

                let mut sig_dst = [0; SIG_LEN];
                buf.copy_to_slice(&mut sig_dst);
                let sig = match sig_from_bytes(Bytes::from(&sig_dst[..])) {
                    Ok(some) => some,
                    Err(_) => {
                        // TODO: Remove malformed msgs
                        return Err(Error::new(ErrorKind::InvalidData, "Invalid Signature"));
                    }
                };
                src.advance(1 + PUBKEY_LEN + SIG_LEN);
                let msg = Message::EndHandshake { pubkey, sig };
                Ok(Some(msg))
            }
            2 => {
                let (nonce_vi, len) = match VarInt::parse_buf(&mut buf) {
                    Ok(Some(some)) => some,
                    Ok(None) => return Ok(None),
                    Err(err) => return Err(Error::new(ErrorKind::Other, err)),
                };

                src.advance(1 + len);
                let msg = Message::Nonce {
                    nonce: u64::from(nonce_vi),
                };
                Ok(Some(msg))
            }
            3 => {
                // let msg_len_vi = match VarInt::parse_buf(&mut buf) {
                //     Ok(some) => some,
                //     Err(_) => return Ok(None),
                // };
                if buf.remaining() < SKETCH_CAPACITY {
                    return Ok(None);
                }
                let mut sketch_dst = [0; SKETCH_CAPACITY];
                buf.copy_to_slice(&mut sketch_dst);
                let msg = Message::OddSketch {
                    sketch: Bytes::from(&sketch_dst[..]),
                };
                src.advance(1 + SKETCH_CAPACITY);
                Ok(Some(msg))
            }
            4 => {
                let (mini_sketch, len) = match DummySketch::parse_buf(&mut buf) {
                    Ok(Some(some)) => some,
                    Ok(None) => return Ok(None),
                    Err(err) => return Err(Error::new(ErrorKind::Other, err)),
                };
                src.advance(1 + len);
                let msg = Message::MiniSketch { mini_sketch };
                Ok(Some(msg))
            }
            5 => {
                println!("Decoding get txns");
                let (n_tx_ids_vi, n_tx_ids_vi_len) = match VarInt::parse_buf(&mut buf) {
                    Ok(Some(some)) => some,
                    Ok(None) => return Ok(None),
                    Err(err) => return Err(Error::new(ErrorKind::Other, err)),
                };
                let us_n_tx_ids = usize::from(n_tx_ids_vi);
                println!("Number of txns to decode {}", us_n_tx_ids);
                let total_size = us_n_tx_ids * TX_ID_LEN;
                let mut ids = HashSet::with_capacity(us_n_tx_ids);

                if buf.remaining() < total_size {
                    Ok(None)
                } else {
                    for _ in 0..us_n_tx_ids {
                        let mut id_dst = [0; TX_ID_LEN];
                        buf.copy_to_slice(&mut id_dst);
                        ids.insert(Bytes::from(&id_dst[..]));
                    }
                    let msg = Message::GetTransactions { ids };
                    src.advance(1 + total_size);
                    Ok(Some(msg))
                }
            }
            6 => {
                let (n_tx_vi, n_tx_vi_len) = match VarInt::parse_buf(&mut buf) {
                    Ok(Some(some)) => some,
                    Ok(None) => return Ok(None),
                    Err(err) => return Err(Error::new(ErrorKind::Other, err)),
                };
                let n_tx = usize::from(n_tx_vi);

                let mut total_size: usize = 0;
                let mut txs = Vec::with_capacity(n_tx);
                for _i in 0..n_tx {
                    let (tx, tx_len) = match Transaction::parse_buf(&mut buf) {
                        Ok(Some(some)) => some,
                        Ok(None) => return Ok(None),
                        Err(err) => return Err(Error::new(ErrorKind::Other, err)),
                    };
                    txs.push(tx);
                    total_size += tx_len;
                }
                let msg = Message::Transactions { txs };
                src.advance(1 + n_tx_vi_len + total_size);
                Ok(Some(msg))
            }
            7 => {
                src.advance(1);
                Ok(Some(Message::Reconcile))
            }
            _ => {
                // TODO: Remove malformed msgs
                println!(
                    "Received: {}",
                    String::from_utf8_lossy(&src.clone().freeze())
                );
                println!("Received: {}", String::from_utf8_lossy(&buf.bytes()));
                Err(Error::new(ErrorKind::InvalidData, "Invalid Message"))
            }
        }
    }
}
