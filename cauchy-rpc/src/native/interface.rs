use std::net::SocketAddr;

use bson::{bson, doc, spec::BinarySubtype, Bson};
use bytes::Bytes;

use failure::Error;
use futures::sync::mpsc::Sender;
use futures::{Future, Stream};
use log::{error, info};
use tokio::codec::Framed;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

use super::messages::RPCCodec;

use crate::{rpc_error, rpc_info, Request, Response};

use core::{
    daemon::{Origin, Priority},
    db::{mongodb::MongoDB, DataType, Database},
    primitives::tx_pool::TxPool,
    utils::{constants::CONFIG, errors::RPCError},
};

pub fn server(
    socket_sender: Sender<TcpStream>,
    stage_send: Sender<(Origin, TxPool, Priority)>,
    db: MongoDB,
) -> Box<Future<Item = (), Error = ()> + Send + 'static> {
    let addr = format!("0.0.0.0:{}", CONFIG.network.rpc_server_port).to_string();
    let addr = addr.parse::<SocketAddr>().unwrap();

    let listener = TcpListener::bind(&addr)
        .map_err(|_| RPCError::BindFailure)
        .unwrap();

    let incoming = listener
        .incoming()
        .map_err(|err| Error::from(RPCError::SocketAcceptanceFailure { err }))
        .map_err(|e| rpc_error!("error accepting socket; error = {:?}", e));

    let server = incoming.for_each(move |socket| {
        let socket_addr = socket.peer_addr().unwrap();
        rpc_info!("new rpc connection to {}", socket_addr);

        // Frame sockets
        let framed_sock = Framed::new(socket, RPCCodec);
        let (send_stream, received_stream) = framed_sock.split();

        // New TCP socket sender
        let socket_sender_inner = socket_sender.clone();
        let db_inner = db.clone();
        let stage_send_inner = stage_send.clone();
        let responses = received_stream.map(move |msg| match msg {
            Request::AddPeer { addr } => {
                rpc_info!("received addpeer {} message from {}", addr, socket_addr);
                let socket_sender_inner = socket_sender_inner.clone();
                tokio::spawn(
                    TcpStream::connect(&addr)
                        .and_then(move |sock| {
                            socket_sender_inner.send(sock).map_err(|_e| {
                                std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "rpc addpeer channel failure",
                                )
                            })
                        })
                        .map(|_| ())
                        .or_else(|e| {
                            rpc_error!("error = {:?}", e);
                            Ok(())
                        }),
                );
                Response::Success
            }
            Request::NewTransaction { tx } => {
                rpc_info!("received new transaction from {}", socket_addr);
                let stage_send_inner = stage_send_inner.clone();
                let mut tx_pool = TxPool::with_capacity(1); // TODO: Make single insertion less clunky
                tx_pool.insert(tx, None, None);
                tokio::spawn(
                    stage_send_inner
                        .send((Origin::RPC, tx_pool, Priority::Standard))
                        .and_then(|_| future::ok(()))
                        .map(|_| ())
                        .or_else(|e| {
                            rpc_error!("error = {:?}", e);
                            Ok(())
                        }),
                );
                Response::Success
            }
            Request::FetchValue { actor_id, key } => {
                let doc = doc! {
                    "t" : Bson::Binary(BinarySubtype::Generic, actor_id.to_vec()),
                    "$or" : [
                        { "p" :  Bson::Null },
                        { "p" : {"$exists" : false}},
                    ],
                    "k" : Bson::Binary(BinarySubtype::Generic, key.to_vec()),
                };
                let result = match db_inner.get(&DataType::State, doc) {
                    Ok(Some(some)) => Bytes::from(&some.get_binary_generic("v").unwrap()[..]),
                    Ok(None) => return Response::NotFound,
                    Err(_) => return Response::Error,
                };
                Response::Value(result)
            }
        });
        let send = send_stream
            .send_all(responses.map_err(|_| RPCError::BindFailure))
            .map(|_| ())
            .or_else(move |e| {
                rpc_error!("socket error {:?}", e);
                Ok(())
            });
        tokio::spawn(send)
    });
    Box::new(server)
}
