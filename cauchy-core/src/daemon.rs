use bytes::Bytes;
use crypto::signatures::ecdsa;
use crypto::sketches::dummy_sketch::*;
use crypto::sketches::odd_sketch::*;
use db::rocksdb::RocksDb;
use db::*;
use futures::sync::mpsc;
use futures::Future;
use net::connections::*;
use net::heartbeats::*;
use net::messages::*;
use net::reconcile_status::*;
use net::rpc_messages::*;
use primitives::arena::Arena;
use primitives::status::Status;
use primitives::transaction::Transaction;
use primitives::varint::VarInt;
use secp256k1::{PublicKey, SecretKey};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::codec::Framed;
use tokio::io::{Error, ErrorKind};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use utils::byte_ops::*;
use utils::constants::*;
use utils::serialisation::*;

pub fn rpc_server(
    tcp_socket_send: mpsc::Sender<TcpStream>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    let addr = format!("0.0.0.0:{}", RPC_SERVER_PORT).to_string();
    let addr = addr.parse::<SocketAddr>().unwrap();

    let listener = TcpListener::bind(&addr)
        .map_err(|_| "failed to bind")
        .unwrap();

    let server = listener
        .incoming()
        .map_err(|e| println!("Error accepting socket; error = {:?}", e))
        .for_each(move |socket| {
            let socket_addr = socket.peer_addr().unwrap();
            if VERBOSE {
                println!("New RPC server socket to {}", socket_addr);
            }

            // Frame sockets
            let framed_sock = Framed::new(socket, RPCCodec);
            let (_, stream) = framed_sock.split();

            // New TCP socket sender
            let tcp_socket_send_inner = tcp_socket_send.clone();

            let send = stream
                .for_each(move |msg| match msg {
                    RPC::AddPeer { addr } => {
                        if VERBOSE {
                            println!("Received addpeer {} message from {}", addr, socket_addr);
                        }
                        let tcp_socket_send_inner = tcp_socket_send_inner.clone();
                        TcpStream::connect(&addr)
                            .and_then(move |sock| {
                                tcp_socket_send_inner.send(sock).map_err(|e| {
                                    Error::new(ErrorKind::Other, "RPC addpeer channel failure")
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

macro_rules! command_peer {
    ($arena: ident, $pk: ident, $fn_name: ident, $input: ident) => {
        let arena_r = $arena.read().unwrap();
        let peer_status = arena_r.get_status(&$pk).unwrap();
        drop(arena_r);
        peer_status.$fn_name($input);
        drop(peer_status);
    };
}

pub fn server(
    tx_db: Arc<RocksDb>,
    local_status: Arc<Status>,
    local_pk: PublicKey,
    local_sk: SecretKey,
    new_stream_recv: mpsc::Receiver<TcpStream>,
    arena: Arc<RwLock<Arena>>,
    connection_manager: Arc<RwLock<ConnectionManager>>,
    rec_status: Arc<RwLock<ReconciliationStatus>>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    // Initialise shared tracking structures arena and connection manager
    /* Arena manages the perceived state of peers and their perception of our local state along
    with proof-of-work calculations */
    /* Connection manager forwards the messages from RPC commands, tracks misbehaviour and handles
    reconciliation/handshake messages */

    let addr = format!("0.0.0.0:{}", SERVER_PORT).to_string();
    let addr = addr.parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr)
        .map_err(|_| "Failed to bind")
        .unwrap();
    let incoming = listener
        .incoming()
        .map_err(|e| println!("Failure accepting socket; {:?}", e))
        .select(new_stream_recv);

    let server = incoming.for_each(move |socket| {
        let socket_addr = socket.peer_addr().unwrap();
        if VERBOSE {
            println!("New server socket to {}", socket_addr);
        }

        // Add new peer with dummy pubkey
        let dummy_pk = ecdsa::generate_dummy_pubkey();
        let socket_pk = Arc::new(RwLock::new(dummy_pk));
        let socket_pk_read = *socket_pk.read().unwrap();
        let mut arena_w = arena.write().unwrap();
        (*arena_w).new_peer(&socket_pk_read);
        drop(arena_w);

        // Pair socket in connection manager
        let mut connection_manager_write_locked = connection_manager.write().unwrap();
        let (secret, msg_receiver) = (*connection_manager_write_locked)
            .add(&socket_addr, socket_pk.clone())
            .unwrap();
        drop(connection_manager_write_locked);

        // Frame the socket
        let framed_sock = Framed::new(socket, MessageCodec);
        let (sink, stream) = framed_sock.split();

        let tx_db_inner = tx_db.clone();

        // Heartbeat OddSketch
        let heartbeat_odd_sketch = heartbeat_oddsketch(
            arena.clone(),
            local_status.clone(),
            rec_status.clone(),
            socket_pk.clone(),
            socket_addr,
        );

        // Heartbeat Nonce
        let heartbeat_nonce = heartbeat_nonce(
            arena.clone(),
            local_status.clone(),
            rec_status.clone(),
            socket_pk.clone(),
            dummy_pk,
            socket_addr,
        );

        // Filter responses
        let socket_pk_inner = socket_pk.clone();
        let arena_inner = arena.clone();
        let rec_status_inner = rec_status.clone();
        let queries = stream.filter(move |msg| match msg {
            Message::StartHandshake { .. } => {
                if VERBOSE {
                    println!("Received handshake initialisation from {}", socket_addr);
                }
                true
            }
            Message::EndHandshake { pubkey, sig } => {
                if VERBOSE {
                    println!("Received handshake finalisation from {}", socket_addr);
                }

                // Add peer to arena
                let secret_msg = ecdsa::message_from_preimage(Bytes::from(VarInt::new(secret)));
                if ecdsa::verify(&secret_msg, sig, pubkey).unwrap() {
                    // If peer correctly signs our secret we upgrade them from a dummy pk
                    if VERBOSE {
                        println!("Handshake completed with {}", socket_addr);
                    }
                    let arena_inner = arena_inner.clone();

                    let mut arena_write = arena_inner.write().unwrap();
                    let socket_pk_read = *socket_pk.read().unwrap();
                    (*arena_write).replace_key(&socket_pk_read, &pubkey);
                    drop(arena_write);
                    let mut socket_pk_write_locked = socket_pk.write().unwrap();
                    *socket_pk_write_locked = *pubkey;
                } else {
                    if VERBOSE {
                        println!("Handshake failed with {}", socket_addr);
                    }
                }
                false
            }
            Message::Nonce { nonce } => {
                if VERBOSE {
                    println!("Received nonce from {}", socket_addr);
                }

                // Update nonce
                let socket_pk_locked = *socket_pk.read().unwrap();
                let nonce = *nonce;
                command_peer!(arena_inner, socket_pk_locked, update_nonce, nonce);
                false
            }
            Message::OddSketch { sketch } => {
                if VERBOSE {
                    println!("Received odd sketch from {}", socket_addr);
                }
                // Update state sketch
                let socket_pk_locked = *socket_pk.read().unwrap();
                let sketch = sketch.clone();
                command_peer!(arena_inner, socket_pk_locked, update_odd_sketch, sketch);
                false
            }
            Message::MiniSketch { .. } => {
                if VERBOSE {
                    println!("Received IBLT from {}", socket_addr);
                }

                // Only response if the pk is reconciliation target
                let socket_pk_read = *socket_pk.read().unwrap();
                rec_status_inner.read().unwrap().eq(&socket_pk_read)
            }
            Message::GetTransactions { .. } => {
                if VERBOSE {
                    println!("Received transaction request from {}", socket_addr);
                }

                true
            }
            Message::Transactions { .. } => {
                if VERBOSE {
                    println!("Received transactions from {}", socket_addr);
                }

                false
            }
            Message::Reconcile => {
                if VERBOSE {
                    println!("Received reconcile from {}", socket_addr);
                }

                true
            }
        });

        let arena_inner = arena.clone();
        let responses = queries.map(move |msg| match msg {
            Message::StartHandshake { secret } => {
                if VERBOSE {
                    println!("Sending handshake finalisation to {}", socket_addr);
                }

                Ok(Message::EndHandshake {
                    pubkey: local_pk,
                    sig: ecdsa::sign(
                        &ecdsa::message_from_preimage(Bytes::from(VarInt::new(secret))),
                        &local_sk,
                    ),
                })
            }
            Message::GetTransactions { ids } => {
                if VERBOSE {
                    println!("Received {} ids", ids.len());
                    println!("Sending transactions to {}", socket_addr);
                }

                let mut txs = Vec::with_capacity(ids.len());
                for id in ids {
                    match tx_db_inner.get(&id) {
                        Ok(Some(tx_raw)) => txs.push(Transaction::try_from(tx_raw).unwrap()),
                        _ => return Err("Couldn't find transaction requested".to_string()),
                    }
                }
                Ok(Message::Transactions { txs })
            }
            Message::MiniSketch { mini_sketch } => {
                if VERBOSE {
                    println!("Sending transactions request to {}", socket_addr);
                }

                let arena_r = arena_inner.read().unwrap();
                let socket_pk_read = *socket_pk_inner.read().unwrap();

                let perception = match (*arena_r).get_perception(&socket_pk_read) {
                    Some(some) => some,
                    None => return Err("No perception found".to_string()),
                };
                let peer_odd_sketch = arena_r
                    .get_status(&socket_pk_read)
                    .unwrap()
                    .get_odd_sketch();

                let perception_sketch = perception.get_mini_sketch();
                let (excess_actor_ids, missing_actor_ids) =
                    (perception_sketch - mini_sketch).decode().unwrap();
                let perception_odd_sketch = perception.get_odd_sketch();
                println!(
                    "Decoding resulted in {} excess and {} missing",
                    excess_actor_ids.len(),
                    missing_actor_ids.len()
                );

                // Check for fraud
                if peer_odd_sketch.byte_xor(perception_odd_sketch)
                    == excess_actor_ids
                        .odd_sketch()
                        .byte_xor(missing_actor_ids.odd_sketch())
                {
                    Ok(Message::GetTransactions {
                        ids: missing_actor_ids,
                    })
                } else {
                    println!("Fraudulent Minisketch");
                    Err("Fraudulent Minisketch provided".to_string())
                }
            }
            Message::Reconcile => {
                if VERBOSE {
                    println!("Sending IBLT to {}", socket_addr);
                }
                let arena_r = arena_inner.read().unwrap();
                let socket_pk_read = *socket_pk_inner.read().unwrap();
                let perception = match (*arena_r).get_perception(&socket_pk_read) {
                    Some(some) => some,
                    None => return Err("No perception found".to_string()),
                };
                println!("Got here");
                Ok(Message::MiniSketch {
                    mini_sketch: perception.get_mini_sketch(),
                })
            }
            _ => unreachable!(),
        });

        // Remove failed responses and merge with heartbeats
        let msg_receiver =
            msg_receiver.map_err(|e| Error::new(ErrorKind::Other, "Message channel failure"));

        let responses = responses.filter(|x| x.is_ok()).map(|x| x.unwrap());
        let out_msgs = responses
            .select(heartbeat_odd_sketch)
            .select(heartbeat_nonce)
            .select(msg_receiver);

        let send = sink.send_all(out_msgs).map(|_| ()).or_else(|e| {
            println!("error = {:?}", e);
            Ok(())
        });
        tokio::spawn(send)
    });
    server
}
