use std::collections::HashSet;
use std::net::SocketAddr;

use bytes::Bytes;
use futures::sync::mpsc::Sender;
use futures::{future, Future, Sink, Stream};
use log::{error, info};
use tokio::codec::Framed;
use tokio::net::{TcpListener, TcpStream};

use core::{
    daemon::{Origin, Priority},
    db::mongodb::MongoDB,
    primitives::transaction::Transaction,
    utils::{constants::config, errors::DaemonError},
};

use super::messages::{RPCCodec, RPC};

use crate::{rpc_error, rpc_info};

fn server(
    socket_sender: Sender<TcpStream>,
    to_stage: Sender<(Origin, HashSet<Transaction>, Priority)>,
    tx_db: MongoDB,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    let addr = format!("0.0.0.0:{}", config.network.rpc_server_port).to_string();
    let addr = addr.parse::<SocketAddr>().unwrap();

    let listener = TcpListener::bind(&addr)
        .map_err(|_| DaemonError::BindFailure)
        .unwrap();

    let server = listener
        .incoming()
        .map_err(|e| rpc_error!("error accepting socket; error = {:?}", e))
        .for_each(move |socket| {
            let socket_addr = socket.peer_addr().unwrap();
            rpc_info!("new rpc connection to {}", socket_addr);

            // Frame sockets
            let framed_sock = Framed::new(socket, RPCCodec);
            let (_, stream) = framed_sock.split();

            // New TCP socket sender
            let socket_sender_inner = socket_sender.clone();
            let to_stage_inner = to_stage.clone();
            let action = stream.map_err(|e| ()).for_each(move |msg| match msg {
                RPC::AddPeer { addr } => {
                    rpc_info!("received addpeer {} message from {}", addr, socket_addr);
                    let socket_sender_inner = socket_sender_inner.clone();
                    tokio::spawn(
                        TcpStream::connect(&addr)
                            .and_then(move |sock| {
                                socket_sender_inner.send(sock).map_err(|e| {
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
                    )
                }
                RPC::NewTransaction { tx } => {
                    rpc_info!("received new transaction from {}", socket_addr);
                    let mut txs = HashSet::new();
                    txs.insert(tx);
                    let to_stage_inner = to_stage_inner.clone();
                    tokio::spawn(
                        to_stage_inner
                            .send((Origin::RPC, txs, Priority::Standard))
                            .and_then(|_| future::ok(()))
                            .map(|_| ())
                            .or_else(|e| {
                                rpc_error!("error = {:?}", e);
                                Ok(())
                            }),
                    )
                }
                RPC::FetchValue { actor_id, key } => unreachable!(),
            });
            tokio::spawn(action)
        });
    server
}
