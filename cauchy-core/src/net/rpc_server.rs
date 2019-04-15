use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::{future, Future, Sink, Stream};
use tokio::codec::Framed;
use tokio::net::{TcpListener, TcpStream};
use futures::sync::mpsc::Sender;

use crate::{
    daemon::{Origin, Priority},
    net::rpc_messages::*,
    primitives::transaction::Transaction,
    utils::{constants::CONFIG, errors::DaemonError},
};

pub fn rpc_server(
    socket_sender: Sender<TcpStream>,
    to_stage: Sender<(Origin, HashSet<Transaction>, Priority)>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    let addr = format!("0.0.0.0:{}", CONFIG.NETWORK.RPC_SERVER_PORT).to_string();
    let addr = addr.parse::<SocketAddr>().unwrap();

    let listener = TcpListener::bind(&addr)
        .map_err(|_| DaemonError::BindFailure)
        .unwrap();

    let server = listener
        .incoming()
        .map_err(|e| println!("error accepting socket; error = {:?}", e))
        .for_each(move |socket| {
            let socket_addr = socket.peer_addr().unwrap();
            if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                println!("new rpc connection to {}", socket_addr);
            }

            // Frame sockets
            let framed_sock = Framed::new(socket, RPCCodec);
            let (_, stream) = framed_sock.split();

            // New TCP socket sender
            let socket_sender_inner = socket_sender.clone();
            let to_stage_inner = to_stage.clone();
            let action = stream.map_err(|e| ()).for_each(move |msg| match msg {
                RPC::AddPeer { addr } => {
                    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                        println!("received addpeer {} message from {}", addr, socket_addr);
                    }
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
                                println!("error = {:?}", e);
                                Ok(())
                            }),
                    )
                }
                RPC::NewTransaction { tx } => {
                    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                        println!("received new transaction from {}", socket_addr);
                    }
                    let mut txs = HashSet::new();
                    txs.insert(tx);
                    let to_stage_inner = to_stage_inner.clone();
                    tokio::spawn(
                        to_stage_inner
                            .send((Origin::RPC, txs, Priority::Standard))
                            .and_then(|_| future::ok(()))
                            .map(|_| ())
                            .or_else(|e| {
                                println!("error = {:?}", e);
                                Ok(())
                            }),
                    )
                }
            });
            tokio::spawn(action)
        });
    server
}
