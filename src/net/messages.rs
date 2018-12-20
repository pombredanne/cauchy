use bytes::Bytes;
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use crypto::signatures::ecdsa::*;
use primitives::transaction::*;
use primitives::varint::VarInt;
use secp256k1::key::PublicKey;
use secp256k1::Signature;
use std::cell::RefCell;
use tokio::codec::{Decoder, Encoder};
use tokio::io::{Error, ErrorKind};
use utils::constants::*;
use utils::serialisation::*;

pub enum Message {
    StartHandshake { preimage: u64 },
    EndHandshake { pubkey: PublicKey, sig: Signature },
    Nonce { nonce: u64 },
    StateSketch { sketch: Bytes },
    GetTransactions { ids: Vec<Bytes> },
    Transactions { txs: Vec<Transaction> },
}

pub struct MessageCodec {
    cached: RefCell<BytesMut>,
}

impl MessageCodec {
    fn new() -> MessageCodec {
        MessageCodec {
            cached: RefCell::new(BytesMut::new()),
        }
    }
}

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            Message::StartHandshake { preimage } => {
                dst.put_u8(0);
                dst.put(Bytes::from(VarInt::new(preimage)));
            }
            Message::EndHandshake { pubkey, sig } => {
                dst.put_u8(1);
                dst.put(bytes_from_pubkey(pubkey));
                dst.put(bytes_from_sig(sig));
            }
            Message::Nonce { nonce } => {
                dst.put_u8(2);
                dst.put_u64_be(nonce);
            }
            Message::StateSketch { sketch } => {
                dst.put_u8(3);
                dst.put(sketch);
            }
            Message::GetTransactions { ids } => {
                dst.put_u8(4);
                dst.put(Bytes::from(VarInt::new(ids.len() as u64)));
                for id in ids {
                    dst.put(id);
                }
            }
            Message::Transactions { txs } => {
                dst.put_u8(5);
                for tx in txs {
                    let raw = Bytes::from(tx);
                    dst.put(Bytes::from(VarInt::new(raw.len() as u64)));
                    dst.put(raw);
                }
            }
        }
        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // TODO: Magic bytes
        let mut cached = self.cached.borrow_mut();
        cached.extend_from_slice(src);
        src.clear();

        let fallback = cached.take();
        let mut buf = fallback.clone().into_buf();

        match buf.get_u8() {
            0 => {
                if buf.remaining() < 8 {
                    cached.extend_from_slice(&fallback);
                    Ok(None)
                } else {
                    let msg = Message::StartHandshake {
                        preimage: u64::from(VarInt::parse_buf(&mut buf)),
                    };
                    cached.extend_from_slice(buf.bytes());
                    Ok(Some(msg))
                }
            }
            1 => {
                if buf.remaining() < PUBKEY_LEN + SIG_LEN {
                    cached.extend_from_slice(&fallback);
                    Ok(None)
                } else {
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
                    cached.extend_from_slice(buf.bytes());
                    let msg = Message::EndHandshake { pubkey, sig };
                    Ok(Some(msg))
                }
            }
            2 => {
                if buf.remaining() < 8 {
                    cached.extend_from_slice(&fallback);
                    Ok(None)
                } else {
                    let msg = Message::Nonce {
                        nonce: buf.get_u64_be(),
                    };
                    cached.extend_from_slice(buf.bytes());
                    Ok(Some(msg))
                }
            }
            3 => {
                if buf.remaining() < SKETCH_LEN {
                    cached.extend_from_slice(&fallback);
                    Ok(None)
                } else {
                    let mut sketch_dst = [0; SKETCH_LEN];
                    buf.copy_to_slice(&mut sketch_dst);
                    let msg = Message::StateSketch {
                        sketch: Bytes::from(&sketch_dst[..]),
                    };
                    cached.extend_from_slice(buf.bytes());
                    Ok(Some(msg))
                }
            }
            4 => {
                let n_vi = VarInt::parse_buf(&mut buf);
                let size = usize::from(n_vi);
                let mut ids = Vec::with_capacity(size);
                if src.len() < TX_ID_LEN * size + 1 {
                    cached.extend_from_slice(&fallback);
                    Ok(None)
                } else {
                    for _i in 0..usize::from(size) {
                        let mut id_dst = [0; TX_ID_LEN];
                        buf.copy_to_slice(&mut id_dst);
                        ids.push(Bytes::from(&id_dst[..]));
                    }
                    let msg = Message::GetTransactions { ids };
                    cached.extend_from_slice(buf.bytes());
                    Ok(Some(msg))
                }
            }
            5 => {
                let n_vi = VarInt::parse_buf(&mut buf);
                let n_tx = u64::from(n_vi);
                let mut txs = Vec::with_capacity(n_tx as usize);
                for _i in 0..n_tx {
                    let tx_len_vi = VarInt::parse_buf(&mut buf);
                    let tx_len = usize::from(tx_len_vi);
                    if buf.remaining() < tx_len {
                        cached.extend_from_slice(&fallback);
                        return Ok(None);
                    } else {
                        match Transaction::parse_buf(&mut buf, tx_len) {
                            Ok(some) => txs.push(some),
                            Err(_) => {
                                cached.extend_from_slice(&fallback);
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "Invalid Transaction",
                                ));
                            }
                        }
                    }
                }
                let msg = Message::Transactions { txs };
                cached.extend_from_slice(buf.bytes());
                Ok(Some(msg))
            }
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid Message")),
        }
    }
}
