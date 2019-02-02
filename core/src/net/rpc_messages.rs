use bytes::{Buf, Bytes, BytesMut, IntoBuf};
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::codec::{Decoder, Encoder};
use tokio::io::{Error, ErrorKind};

pub struct RPCCodec;

pub enum RPC {
    AddPeer { addr: SocketAddr }, // 0 || Peer addr
}

impl Encoder for RPCCodec {
    type Item = RPC;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        Err(Error::new(
            ErrorKind::Other,
            "Daemon shouldn't send RPC messages",
        ))
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
            _ => unreachable!(),
        }
    }
}
