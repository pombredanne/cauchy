use std::net::SocketAddr;

use bytes::{Buf, BytesMut, IntoBuf};
use failure::Error;
use tokio::codec::{Decoder, Encoder};
// use tokio::io::{Error, ErrorKind};

use crate::primitives::transaction::Transaction;
use crate::utils::parsing::Parsable;

pub struct RPCCodec;

pub enum RPC {
    AddPeer { addr: SocketAddr },       // 0 || Peer addr
    NewTransaction { tx: Transaction }, // 1 || Transaction
}

impl Encoder for RPCCodec {
    type Item = RPC;
    type Error = Error;

    fn encode(&mut self, _item: Self::Item, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        unreachable!() // TODO: Return values
    }
}

impl Decoder for RPCCodec {
    type Item = RPC;
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
                Ok(Some(RPC::AddPeer { addr }))
            }
            1 => {
                // Add transaction to own state
                let (tx, _) = match Transaction::parse_buf(&mut buf)? {
                    Some(some) => some,
                    None => return Ok(None),
                };

                Ok(Some(RPC::NewTransaction { tx }))
            }
            _ => unreachable!(),
        }
    }
}
