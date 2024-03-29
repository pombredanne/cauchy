use std::net::SocketAddr;

use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use failure::Error;
use tokio::codec::{Decoder, Encoder};
// use tokio::io::{Error, ErrorKind};

use core::primitives::transaction::Transaction;
use core::utils::constants::HASH_LEN;
use core::utils::parsing::Parsable;

use crate::{Request, Response};

pub struct RPCCodec;

impl Encoder for RPCCodec {
    type Item = Response;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            Response::Success => {
                dst.put_u8(0);
            }
            Response::Error => {
                dst.put_u8(1);
            } // TODO: Add error msg
            Response::NotFound => {
                dst.put_u8(2);
            }
            Response::Value(val) => {
                dst.put_u8(2);
                dst.put_u32_be(val.len() as u32);
                dst.extend(val);
            }
        }
        Ok(())
    }
}

impl Decoder for RPCCodec {
    type Item = Request;
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut buf = src.clone().into_buf();

        if buf.remaining() == 0 {
            return Ok(None);
        }

        match buf.get_u8() {
            0 => {
                // Add Peer
                // TODO: Catch errors
                let mut dst_ip = [0; 4];
                buf.copy_to_slice(&mut dst_ip);
                let dst_port = buf.get_u16_be();
                let addr = SocketAddr::from((dst_ip, dst_port));
                src.advance(7);
                Ok(Some(Request::AddPeer { addr }))
            }
            1 => {
                // Add transaction to own state
                let (tx, tx_len) = match Transaction::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };
                src.advance(tx_len + 1);
                Ok(Some(Request::NewTransaction { tx }))
            }
            2 => {
                // Add transaction to own state
                let mut dst_actor_id = [0; HASH_LEN];
                buf.copy_to_slice(&mut dst_actor_id);

                let mut dst_key = [0; HASH_LEN];
                buf.copy_to_slice(&mut dst_key);

                src.advance(2 * HASH_LEN + 1);
                Ok(Some(Request::FetchValue {
                    actor_id: Bytes::from(&dst_actor_id[..]),
                    key: Bytes::from(&dst_key[..]),
                }))
            }
            _ => unreachable!(),
        }
    }
}
