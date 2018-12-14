use bytes::{Buf, BufMut, Bytes, IntoBuf};
use crypto::signatures::ecdsa::*;
use primitives::transaction::Transaction;
use primitives::varint::VarInt;
use secp256k1::key::PublicKey;
use secp256k1::Signature;
use utils::constants::*;
use utils::serialisation::*;

pub enum Message {
    StartHandshake { preimage: Bytes },
    EndHandshake { pubkey: PublicKey, sig: Signature },
    GetTransaction { tx_id: Bytes },
    Transaction { tx: Transaction },
    GetStateSketch,
    StateSketch { sketch: Bytes },
    GetNonce,
    Nonce { nonce: u64 },
    Error { msg: String },
    End,
}

impl Message {
    pub fn parse(input: Bytes) -> Result<Message, String> {
        let mut buf = input.into_buf();
        let classifier = buf.get_u8();
        match classifier {
            0 => Ok(Message::StartHandshake {
                preimage: Bytes::from(buf.bytes()),
            }),
            1 => {
                let mut dst_a = vec![0; PUBKEY_LEN];
                let mut dst_b = vec![0; SIG_LEN];

                buf.copy_to_slice(&mut dst_a);
                buf.copy_to_slice(&mut dst_b);

                let publickey = match pubkey_from_bytes(Bytes::from(dst_a)) {
                    Ok(pk) => pk,
                    Err(error) => return Err(error),
                };
                let signature = match sig_from_bytes(Bytes::from(dst_b)) {
                    Ok(sig) => sig,
                    Err(error) => return Err(error),
                };
                Ok(Message::EndHandshake {
                    pubkey: publickey,
                    sig: signature,
                })
            }
            2 => {
                let mut dst = vec![0; TX_ID_LEN];
                buf.copy_to_slice(&mut dst);
                Ok(Message::GetTransaction {
                    tx_id: Bytes::from(dst),
                })
            }
            3 => {
                let transaction = match Transaction::try_from(Bytes::from(buf.bytes())) {
                    Ok(tx) => tx,
                    Err(err) => return Err(err),
                };
                Ok(Message::Transaction { tx: transaction })
            }
            4 => Ok(Message::GetStateSketch),
            5 => Ok(Message::StateSketch {
                sketch: Bytes::from(buf.bytes()),
            }),
            6 => Ok(Message::GetNonce),
            7 => {
                let nonce = match VarInt::try_from(Bytes::from(buf.bytes())) {
                    Ok(vi) => u64::from(vi),
                    Err(err) => return Err(err),
                };
                Ok(Message::Nonce { nonce })
            }
            _ => {
                let msg = buf.bytes();
                Err(String::from_utf8_lossy(msg).into_owned())
            }
        }
    }

    pub fn serialise(self) -> Bytes {
        let mut buf = vec![];
        match self {
            Message::StartHandshake { preimage } => {
                buf.put_u8(0);
                buf.put_slice(&preimage);
            }
            Message::EndHandshake { pubkey, sig } => {
                buf.put_u8(1);
                buf.put_slice(&bytes_from_pubkey(pubkey));
                buf.put_slice(&bytes_from_sig(sig));
            }
            Message::GetTransaction { tx_id } => {
                buf.put_u8(2);
                buf.put_slice(&tx_id);
            }
            Message::Transaction { tx } => {
                buf.put_u8(3);
                buf.put_slice(&Bytes::from(tx));
            }
            Message::GetStateSketch => {
                buf.put_u8(4);
            }
            Message::StateSketch { sketch } => {
                buf.put_u8(5);
                println!("{}", sketch.len());
                buf.put_slice(&sketch);
            }
            Message::GetNonce => {
                buf.put_u8(6);
            }
            Message::Nonce { nonce } => {
                buf.put_u8(7);
                buf.put_slice(&Bytes::from(VarInt::new(nonce)));
            }
            Message::Error { msg } => buf.put_slice(&msg.as_bytes()[..]),
            _ => (),
        }
        Bytes::from(buf)
    }
}
