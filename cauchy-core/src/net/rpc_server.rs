use futures::{Future, Sink, Stream};
use net::rpc_messages::*;
use std::net::SocketAddr;
use tokio::codec::Framed;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;

use utils::constants::{CONFIG, DAEMON_VERBOSE};
use utils::errors::DaemonError;

pub fn rpc_server(
    socket_sender: Sender<TcpStream>,
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
            if DAEMON_VERBOSE {
                println!("New RPC server socket to {}", socket_addr);
            }

            // Frame sockets
            let framed_sock = Framed::new(socket, RPCCodec);
            let (_, stream) = framed_sock.split();

            // New TCP socket sender
            let socket_sender_inner = socket_sender.clone();
            let action = stream
                .for_each(move |msg| match msg {
                    RPC::AddPeer { addr } => {
                        if DAEMON_VERBOSE {
                            println!("Received addpeer {} message from {}", addr, socket_addr);
                        }
                        let socket_sender_inner = socket_sender_inner.clone();
                        TcpStream::connect(&addr)
                            .and_then(move |sock| {
                                socket_sender_inner.send(sock).map_err(|e| {
                                    std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        "RPC addpeer channel failure",
                                    )
                                })
                            })
                            .map(|_| ())
                            .or_else(|e| {
                                println!("error = {:?}", e);
                                Ok(())
                            })
                    }
                })
                .map_err(|e| ());
            tokio::spawn(action)
        });
    server
}
