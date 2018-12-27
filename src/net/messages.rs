use bytes::Bytes;
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use crypto::signatures::ecdsa::*;
use primitives::transaction::*;
use primitives::varint::VarInt;
use secp256k1::key::PublicKey;
use secp256k1::Signature;
use tokio::codec::{Decoder, Encoder};
use tokio::io::{Error, ErrorKind};
use utils::constants::*;

pub enum Message {
    StartHandshake { secret: u64 },
    EndHandshake { pubkey: PublicKey, sig: Signature },
    Nonce { nonce: u64 },
    StateSketch { sketch: Bytes },
    GetTransactions { ids: Vec<Bytes> },
    Transactions { txs: Vec<Transaction> },
}

pub struct MessageCodec;

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // TODO: Manage capacity better
        let mut payload = BytesMut::new();
        match item {
            Message::StartHandshake { secret } => {
                payload.put_u8(0);
                payload.extend(Bytes::from(VarInt::new(secret)));

                dst.extend(Bytes::from(VarInt::new(payload.len() as u64)));
                dst.extend(payload);
            }
            Message::EndHandshake { pubkey, sig } => {
                payload.put_u8(1);
                payload.extend(bytes_from_pubkey(pubkey));
                payload.extend(bytes_from_sig(sig));

                dst.extend(Bytes::from(VarInt::new(payload.len() as u64)));
                dst.extend(payload);
            }
            Message::Nonce { nonce } => {
                //println!("Send nonce: {}", nonce);
                payload.put_u8(2);
                payload.extend(Bytes::from(VarInt::new(nonce)));

                dst.extend(Bytes::from(VarInt::new(payload.len() as u64)));
                dst.extend(payload);
            }
            Message::StateSketch { sketch } => {
                payload.put_u8(3);
                payload.extend(sketch);

                dst.extend(Bytes::from(VarInt::new(payload.len() as u64)));
                dst.extend(payload);
            }
            Message::GetTransactions { ids } => {
                payload.put_u8(4);
                payload.extend(Bytes::from(VarInt::new(ids.len() as u64)));
                for id in ids {
                    payload.extend(id);
                }

                dst.extend(Bytes::from(VarInt::new(payload.len() as u64)));
                dst.extend(payload);
            }
            Message::Transactions { txs } => {
                payload.put_u8(5);
                for tx in txs {
                    let raw = Bytes::from(tx);
                    payload.extend(Bytes::from(VarInt::new(raw.len() as u64)));
                    payload.extend(raw);
                }

                dst.extend(Bytes::from(VarInt::new(payload.len() as u64)));
                dst.extend(payload);
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

        let mut buf = src.clone().into_buf();

        let msg_len_vi = match VarInt::parse_buf(&mut buf) {
            Ok(some) => some,
            Err(_) => return Ok(None),
        };
        let msg_len_len = msg_len_vi.len();
        let msg_len = usize::from(msg_len_vi);

        if buf.remaining() < msg_len {
            return Ok(None);
        }

        src.advance(msg_len_len);

        match buf.get_u8() {
            0 => {
                let preimage_vi = match VarInt::parse_buf(&mut buf) {
                    Ok(some) => some,
                    Err(_) => return Err(Error::new(ErrorKind::Other, "Failed to parse VarInt")),
                };
                let msg = Message::StartHandshake {
                    secret: u64::from(preimage_vi),
                };
                src.advance(msg_len);
                Ok(Some(msg))
            }
            1 => {
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
                let msg = Message::EndHandshake { pubkey, sig };
                src.advance(msg_len);
                Ok(Some(msg))
            }
            2 => {
                let nonce_vi = match VarInt::parse_buf(&mut buf) {
                    Ok(some) => some,
                    Err(_) => return Err(Error::new(ErrorKind::Other, "Failed to parse VarInt")),
                };
                let msg = Message::Nonce {
                    nonce: u64::from(nonce_vi),
                };
                src.advance(msg_len);
                Ok(Some(msg))
            }
            3 => {
                let mut sketch_dst = [0; SKETCH_LEN];
                buf.copy_to_slice(&mut sketch_dst);
                let msg = Message::StateSketch {
                    sketch: Bytes::from(&sketch_dst[..]),
                };
                src.advance(msg_len);
                Ok(Some(msg))
            }
            4 => {
                let n_vi = match VarInt::parse_buf(&mut buf) {
                    Ok(some) => some,
                    Err(_) => return Err(Error::new(ErrorKind::Other, "Failed to parse VarInt")),
                };
                let size = usize::from(n_vi);
                let mut ids = Vec::with_capacity(size);
                if src.len() < TX_ID_LEN * size + 1 {
                    Ok(None)
                } else {
                    for _i in 0..usize::from(size) {
                        let mut id_dst = [0; TX_ID_LEN];
                        buf.copy_to_slice(&mut id_dst);
                        ids.push(Bytes::from(&id_dst[..]));
                    }
                    let msg = Message::GetTransactions { ids };
                    src.advance(msg_len);
                    Ok(Some(msg))
                }
            }
            5 => {
                let n_vi = match VarInt::parse_buf(&mut buf) {
                    Ok(some) => some,
                    Err(_) => return Err(Error::new(ErrorKind::Other, "Failed to parse VarInt")),
                };
                let n_tx = u64::from(n_vi);
                let mut txs = Vec::with_capacity(n_tx as usize);
                for _i in 0..n_tx {
                    let tx_len_vi = match VarInt::parse_buf(&mut buf) {
                        Ok(some) => some,
                        Err(_) => {
                            return Err(Error::new(ErrorKind::Other, "Failed to parse VarInt"))
                        }
                    };
                    let tx_len = usize::from(tx_len_vi);
                    if buf.remaining() < tx_len {
                        return Ok(None);
                    } else {
                        match Transaction::parse_buf(&mut buf, tx_len) {
                            Ok(some) => txs.push(some),
                            Err(_) => {
                                // TODO: Remove malformed msgs
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "Invalid Transaction",
                                ));
                            }
                        }
                    }
                }
                let msg = Message::Transactions { txs };
                src.advance(msg_len);
                Ok(Some(msg))
            }
            _ => {
                // TODO: Remove malformed msgs
                println!("HELLO");
                println!("{}", String::from_utf8_lossy(&src.clone().freeze()));
                println!("{}", String::from_utf8_lossy(&buf.bytes()));
                Err(Error::new(ErrorKind::InvalidData, "Invalid Message"))
            }
        }
    }
}
