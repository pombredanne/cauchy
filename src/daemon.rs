use bytes::Bytes;
use db::rocksdb::RocksDb;
use db::*;
use primitives::status::Status;
use secp256k1::{PublicKey, SecretKey};

use net::messages::*;
use net::rpc_messages::*;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::codec::Framed;
use tokio::io::{Error, ErrorKind};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::timer::Interval;

use crypto::signatures::ecdsa;
use crypto::sketches::iblt::*;
use crypto::sketches::odd_sketch::*;
use primitives::arena::Arena;
use primitives::transaction::Transaction;
use primitives::varint::VarInt;
use utils::byte_ops::*;
use utils::constants::*;
use utils::serialisation::*;

pub fn rpc_server(secret: Arc<RwLock<u64>>, verbose: bool) {
    let addr = env::args().nth(1).unwrap_or(format!("0.0.0.0:{}", RPC_SERVER_PORT).to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let listener = TcpListener::bind(&addr)
        .map_err(|_| "failed to bind")
        .unwrap();

    let done = listener
        .incoming()
        .map_err(|e| println!("error accepting socket; error = {:?}", e))
        .for_each(move |socket| {
            let socket_addr = socket.peer_addr().unwrap();
            if verbose { println!("New RPC server socket to {}", socket_addr); }

            let framed_sock = Framed::new(socket, RPCCodec);
            let (_, stream) = framed_sock.split();
            let secret_c = secret.clone();

            stream.for_each(move |msg| match msg {
                RPC::AddPeer { addr } => {
                    // TODO: Catch errors
                    if verbose { println!("Received addpeer {} message from {}", addr, socket_addr); }

                    let secret_c = secret_c.clone();
                    TcpStream::connect(&addr)
                        .and_then(move |sock| {
                            if verbose { println!("Sent handshake initialisation to {}", addr); }

                            let framed_sock = Framed::new(sock, MessageCodec);
                            let secret_r: u64 = *secret_c.read().unwrap();
                            framed_sock.send(Message::StartHandshake { secret: secret_r }).poll();
                            Ok(())
                        })
                }
            }).map(|_| ()).or_else(|e| {
                println!("error = {:?}", e);
                Ok(())
            })
        });
        tokio::run(done);
}

pub fn server(
    tx_db: Arc<RocksDb>,
    self_status: Arc<Status>,
    local_pk: PublicKey,
    local_sk: SecretKey,
    secret: Arc<RwLock<u64>>,
    verbose: bool,
) {
    let mut arena = Arc::new(RwLock::new(Arena::new(&local_pk, self_status.clone())));

    let addr = env::args().nth(1).unwrap_or(format!("0.0.0.0:{}", SERVER_PORT).to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let dummy_pk = ecdsa::generate_dummy_pubkey();

    let listener = TcpListener::bind(&addr)
        .map_err(|_| "failed to bind")
        .unwrap();

    let done = listener
        .incoming()
        .map_err(|e| println!("error accepting socket; error = {:?}", e))
        .for_each(move |socket| {
            let socket_addr = socket.peer_addr().unwrap();
            if verbose { println!("New server socket to {}", socket_addr); }

            let socket_pk = Arc::new(RwLock::new(dummy_pk));
            let framed_sock = Framed::new(socket, MessageCodec);
            let (sink, stream) = framed_sock.split();
            let tx_db_c = tx_db.clone();

            // Heartbeat OddSketch
            let arena_c_a = arena.clone();
            let arena_c_b = arena.clone();
            let self_status_c = self_status.clone();
            let heartbeat_odd_sketch = Interval::new_interval(Duration::new(
                ODDSKETCH_HEARTBEAT_PERIOD_SEC,
                ODDSKETCH_HEARTBEAT_PERIOD_NANO,
            ))
            .map(move |_| {
                if verbose { println!("Sending odd sketch to {}", socket_addr); }
                Message::OddSketch {
                sketch: self_status_c.get_odd_sketch(),
            }})
            .map_err(|e| Error::new(ErrorKind::Other, "Odd sketch heart failure"));

            // Heartbeat Nonce
            let self_status_c = self_status.clone();
            let socket_pk_c = socket_pk.clone();
            let heartbeat_nonce = Interval::new_interval(Duration::new(
                NONCE_HEARTBEAT_PERIOD_SEC,
                NONCE_HEARTBEAT_PERIOD_NANO,
            ))
            .map(move |_| (self_status_c.get_nonce(), *socket_pk_c.read().unwrap()))
            .filter(move |(_, sock_pk)| *sock_pk != dummy_pk)
            .filter(move |(current_nonce, sock_pk)| {
                *current_nonce != (*arena_c_a.read().unwrap()).get_perception(sock_pk).nonce
            })
            .map(move |(current_nonce, sock_pk)| {
                if verbose { println!("Sending nonce to {}", socket_addr); }
                let mut arena_r = arena_c_b.write().unwrap();
                arena_r.update_perception(&sock_pk);

                Message::Nonce {
                    nonce: current_nonce,
                }
            })
            .map_err(|e| Error::new(ErrorKind::Other, "Nonce heart failure"));

            // Heartbeat Reconcile
            // TODO: Move out of this scope
            let socket_pk_c = socket_pk.clone();
            let arena_c = arena.clone();
            let heartbeat_reconcile = Interval::new_interval(Duration::new(
                RECONCILE_HEARTBEAT_PERIOD_SEC,
                RECONCILE_HEARTBEAT_PERIOD_NANO,
            ))
            .map(move |_| *socket_pk_c.read().unwrap())
            .filter(move |sock_pk| *sock_pk != dummy_pk)
            .filter(move |sock_pk| {
                if verbose { println!("Sending reconcile to {}", socket_addr); }

                // Update order
                let mut arena_r = arena_c.write().unwrap();
                arena_r.update_order();
                drop(arena_r);

                // Find leader
                let arena_r = arena_c.read().unwrap();
                let leader_pk = arena_r.get_order()[0];
                println!("Leader: {}", leader_pk);
                println!("Socket: {}", sock_pk);
                *sock_pk == leader_pk
            })
            .map(|_| Message::Reconcile)
            .map_err(|e| Error::new(ErrorKind::Other, e));

            // Filter responses
            let socket_pk_c = socket_pk.clone();
            let arena_c = arena.clone();
            let secret_c = secret.clone();
            let queries = stream.filter(move |msg| match msg {
                Message::StartHandshake { .. } => {
                    if verbose { println!("Received handshake initialisation from {}", socket_addr); }
                    true
                    },
                Message::EndHandshake { pubkey, sig } => {
                    if verbose { println!("Received handshake finalisation from {}", socket_addr); }

                    // Add peer to arena
                    let new_status = Arc::new(Status::null());
                    let mut arena_m = arena_c.write().unwrap();
                    let secret_r: u64 = *secret_c.read().unwrap();
                    let secret_msg =
                        ecdsa::message_from_preimage(Bytes::from(VarInt::new(secret_r)));
                    if ecdsa::verify(&secret_msg, sig, pubkey).unwrap() {
                        arena_m.add_peer(&pubkey, new_status);
                        let mut socket_pk_locked = socket_pk_c.write().unwrap();
                        *socket_pk_locked = *pubkey;
                    }
                    false
                }
                Message::Nonce { nonce } => {
                    if verbose { println!("Received nonce from {}", socket_addr); }

                    // Update nonce
                    let arena_r = arena_c.read().unwrap();
                    let socket_pk_locked = *socket_pk_c.read().unwrap();

                    let peer_status = arena_r.get_peer(&socket_pk_locked);
                    peer_status.update_nonce(*nonce);
                    false
                }
                Message::OddSketch { sketch } => {
                    if verbose { println!("Received odd sketch from {}", socket_addr); }

                    // Update state sketch
                    let arena_r = arena_c.read().unwrap();
                    let socket_pk_locked = socket_pk_c.read().unwrap();
                    let peer_status = arena_r.get_peer(&*socket_pk_locked);
                    peer_status.update_odd_sketch(sketch.clone());
                    false
                }
                Message::IBLT { iblt } => {
                    if verbose { println!("Received IBLT from {}", socket_addr); }

                    let arena_r = arena_c.read().unwrap();
                    let socket_pk_locked = *socket_pk_c.read().unwrap();
                    let peer_status = arena_r.get_peer(&socket_pk_locked);
                    peer_status.update_mini_sketch(iblt.clone());
                    true
                }
                Message::GetTransactions { .. } => {
                    if verbose { println!("Received transaction request from {}", socket_addr); }

                    true },
                Message::Transactions { .. } => {
                    if verbose { println!("Received transactions from {}", socket_addr); }

                     false },
                Message::Reconcile => {
                    if verbose { println!("Received reconcile from {}", socket_addr); }

                     true },
            });

            let arena_c = arena.clone();
            let responses = queries.map(move |msg| match msg {
                Message::StartHandshake { secret } => {
                    if verbose { println!("Sending handshake finalisation to {}", socket_addr); }

                    Ok(Message::EndHandshake {
                    pubkey: local_pk,
                    sig: ecdsa::sign(
                        &ecdsa::message_from_preimage(Bytes::from(VarInt::new(secret))),
                        &local_sk,
                    ),
                })
                },
                Message::GetTransactions { ids } => {
                    if verbose { println!("Sending transactions to {}", socket_addr); }

                    let mut txs = Vec::with_capacity(ids.len());
                    for id in ids {
                        match tx_db_c.get(&id) {
                            Ok(Some(tx_raw)) => txs.push(Transaction::try_from(tx_raw).unwrap()),
                            _ => (),
                        }
                    }
                    Ok(Message::Transactions { txs })
                }
                Message::IBLT { iblt } => {
                    if verbose { println!("Sending transactions request to {}", socket_addr); }

                    let arena_r = arena_c.read().unwrap();
                    let socket_pk_locked = *socket_pk.read().unwrap();

                    let perception = arena_r.get_perception(&socket_pk_locked);

                    let perception_iblt: IBLT = perception.mini_sketch;
                    let (ids, _) = (perception_iblt - iblt).decode().unwrap();
                    let peer_odd_sketch = arena_r.get_peer(&socket_pk_locked).get_odd_sketch();
                    let perception_odd_sketch = perception.odd_sketch;

                    // Check for fraud
                    if peer_odd_sketch.byte_xor(perception_odd_sketch) == ids.odd_sketch() {
                        Ok(Message::GetTransactions { ids })
                    } else {
                        Err("Fraudulent Minisketch provided".to_string())
                    }
                }
                Message::Reconcile => {
                    if verbose { println!("Sending IBLT to {}", socket_addr); }

                    let iblt = (*arena_c.read().unwrap())
                        .get_peer(&local_pk)
                        .get_mini_sketch();
                    Ok(Message::IBLT { iblt })
                }
                _ => unreachable!(),
            });

            // Remove failed responses and merge with heartbeats
            let responses = responses.filter(|x| x.is_ok()).map(|x| x.unwrap());
            let out_msgs = responses
                .select(heartbeat_odd_sketch)
                .select(heartbeat_nonce)
                .select(heartbeat_reconcile);

            sink.send_all(out_msgs).map(|_| ()).or_else(|e| {
                println!("error = {:?}", e);
                Ok(())
            })
        });
    tokio::run(done);
}
