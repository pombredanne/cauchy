use net::rpc_messages::*;
use tokio::net::{TcpListener, TcpStream};
use utils::constants::{RPC_SERVER_PORT, DAEMON_VERBOSE};
use std::net::SocketAddr;
use futures::{Future, Stream, Sink};
use utils::errors::DaemonError;
use tokio::codec::Framed;
use net::connections::ConnectionManager;
use std::sync::{Arc, RwLock};

pub fn rpc_server(
    connection_manager: Arc<RwLock<ConnectionManager>>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    let addr = format!("0.0.0.0:{}", RPC_SERVER_PORT).to_string();
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
            let tcp_socket_send_inner = connection_manager.read().unwrap().get_new_socket_send();

            let send = stream
                .for_each(move |msg| match msg {
                    RPC::AddPeer { addr } => {
                        if DAEMON_VERBOSE {
                            println!("Received addpeer {} message from {}", addr, socket_addr);
                        }
                        let tcp_socket_send_inner = tcp_socket_send_inner.clone();
                        TcpStream::connect(&addr)
                            .and_then(move |sock| {
                                tcp_socket_send_inner.send(sock).map_err(|e| {
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
            tokio::spawn(send)
        });
    server
}