use bytes::Bytes;
use crypto::signatures::ecdsa;
use crypto::sketches::odd_sketch::*;
use crypto::sketches::*;
use db::rocksdb::RocksDb;
use db::storing::Storable;
use failure::Error;
use futures::sync::mpsc;
use futures::Future;
use net::connections::*;
use net::heartbeats::*;
use net::messages::*;
use net::reconcile_status::*;
use primitives::arena::Arena;
use primitives::status::Status;
use primitives::transaction::Transaction;
use primitives::varint::VarInt;
use secp256k1::{PublicKey, SecretKey};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::codec::Framed;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use utils::constants::*;
use utils::errors::DaemonError;

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
        .map_err(|_| DaemonError::BindFailure)
        .unwrap();
    let incoming = listener
        .incoming()
        .map_err(|err| Error::from(DaemonError::SocketAcceptanceFailure { err }))
        .select(new_stream_recv.map_err(|err| Error::from(DaemonError::Unreachable)))
        .map_err(|e| println!("error accepting socket; error = {:?}", e));

    let server = incoming.for_each(move |socket| {
        let socket_addr = socket.peer_addr().unwrap();
        if DAEMON_VERBOSE {
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
        let (secret, impulse_recv) = connection_manager_write_locked
            .add(&socket_addr, socket_pk.clone())
            .unwrap();
        drop(connection_manager_write_locked);

        // Frame the socket
        let framed_sock = Framed::new(socket, MessageCodec);
        let (sink, received_stream) = framed_sock.split();

        // Heartbeat OddSketch
        let odd_sketch_stream = heartbeat_oddsketch(
            arena.clone(),
            local_status.clone(),
            rec_status.clone(),
            socket_pk.clone(),
            socket_addr,
        );

        // Heartbeat Nonce
        let nonce_stream = heartbeat_nonce(
            arena.clone(),
            local_status.clone(),
            rec_status.clone(),
            socket_pk.clone(),
            dummy_pk,
            socket_addr,
        );

        // Filter through received messages
        let socket_pk_inner = socket_pk.clone();
        let arena_inner = arena.clone();
        let rec_status_inner = rec_status.clone();
        let local_status_inner = local_status.clone();
        let received_stream = received_stream.filter(move |msg| match msg {
            Message::StartHandshake { .. } => {
                if DAEMON_VERBOSE {
                    println!("Received handshake initialisation from {}", socket_addr);
                }
                true
            }
            Message::EndHandshake { pubkey, sig } => {
                if DAEMON_VERBOSE {
                    println!("Received handshake finalisation from {}", socket_addr);
                }

                // Add peer to arena
                let secret_msg = ecdsa::message_from_preimage(Bytes::from(VarInt::new(secret)));
                if ecdsa::verify(&secret_msg, sig, pubkey).unwrap() {
                    if DAEMON_VERBOSE {
                        println!("Handshake completed with {}", socket_addr);
                    }
                    // If peer correctly signs our secret we upgrade them from a dummy pk
                    let arena_inner = arena_inner.clone();

                    let mut arena_write = arena_inner.write().unwrap();
                    let socket_pk_read = *socket_pk.read().unwrap();
                    (*arena_write).replace_key(&socket_pk_read, &pubkey);
                    drop(arena_write);
                    let mut socket_pk_write_locked = socket_pk.write().unwrap();
                    *socket_pk_write_locked = *pubkey;
                } else {
                    if DAEMON_VERBOSE {
                        println!("Handshake failed with {}", socket_addr);
                    }
                }
                false
            }
            Message::Nonce { nonce } => {
                if DAEMON_VERBOSE {
                    println!("Received nonce from {}", socket_addr);
                }

                // Update nonce
                let socket_pk_locked = *socket_pk.read().unwrap();
                let nonce = *nonce;
                command_peer!(arena_inner, socket_pk_locked, update_nonce, nonce);
                false
            }
            Message::OddSketch { sketch } => {
                if DAEMON_VERBOSE {
                    println!("Received odd sketch from {}", socket_addr);
                }
                // Update state sketch
                let socket_pk_locked = *socket_pk.read().unwrap();
                let sketch = sketch.clone();
                command_peer!(arena_inner, socket_pk_locked, update_odd_sketch, sketch);
                false
            }
            Message::MiniSketch { .. } => {
                if DAEMON_VERBOSE {
                    println!("Received MiniSketch from {}", socket_addr);
                }

                // Only response if the pk is reconciliation target
                let socket_pk_read = *socket_pk.read().unwrap();
                rec_status_inner.read().unwrap().target_eq(&socket_pk_read)
            }
            Message::GetTransactions { .. } => {
                // TODO: Check if reconcilee?
                if DAEMON_VERBOSE {
                    println!("Received transaction request from {}", socket_addr);
                }
                true
            }
            Message::Transactions { txs } => {
                if DAEMON_VERBOSE {
                    println!("Received transactions from {}", socket_addr);
                }
                // If received txs from reconciliation target check the payload matches reported
                // TODO: IDs should be calculated before we read to reduce unnecesarry concurrency on rec_status?
                let socket_pk_read = *socket_pk.read().unwrap();
                let rec_status_read = rec_status_inner.read().unwrap();
                if rec_status_read.target_eq(&socket_pk_read) {
                    if DAEMON_VERBOSE {
                        println!("Checking payload IDs match requested");
                    }
                    if rec_status_read.missing_ids_eq(&txs) {
                        if DAEMON_VERBOSE {
                            println!("Payload is valid.");
                        }
                        // TODO: Send side stage for validation

                        // TODO: Update state, this is here temporarily
                        let arena_r = arena_inner.read().unwrap();
                        let perception = match arena_r.get_perception(&socket_pk_read) {
                            Some(some) => some,
                            None => return false,
                        };
                        rec_status_read.final_update(local_status_inner.clone(), perception)

                    } else {
                        if DAEMON_VERBOSE {
                            println!("Payload is invalid.");
                        }
                        // TODO: Increment banscore
                    }
                    drop(rec_status_read);
                    rec_status_inner.write().unwrap().stop();
                } else {
                    // TODO: Send to acceptor
                }
                false
            }
            Message::Reconcile => {
                if DAEMON_VERBOSE {
                    println!("Received reconcile from {}", socket_addr);
                }
                true
            }
        });

        // Construct responses
        let arena_inner = arena.clone();
        let tx_db_inner = tx_db.clone();
        let rec_status_inner = rec_status.clone();
        let response_stream = received_stream.map(move |msg| match msg {
            Message::StartHandshake { secret } => {
                if DAEMON_VERBOSE {
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
                if DAEMON_VERBOSE {
                    println!("Received {} ids", ids.len());
                }

                // Remove reconcilee
                let socket_pk_read = *socket_pk_inner.read().unwrap();
                rec_status_inner.write().unwrap().remove_reconcilee(&socket_pk_read);

                let mut txs = HashSet::with_capacity(ids.len());
                for id in ids {
                    if DAEMON_VERBOSE {
                        println!("Searching for transaction {:?}", id);
                    }
                    let tx_db_inner = tx_db_inner.clone();
                    match Transaction::from_db(tx_db_inner, &id) {
                        Ok(Some(tx)) => {
                            // if DAEMON_VERBOSE {
                            //     println!("Found {:?}", id);
                            // }
                            txs.insert(tx);
                        }
                        Err(err) => {
                            if DAEMON_VERBOSE {
                                println!("Database error {:?}", err);
                            }
                            return Err(err);
                        }
                        Ok(None) => {
                            if DAEMON_VERBOSE {
                                println!("Transaction {:?} not found", id);
                            }
                            return Err(DaemonError::MissingTransaction.into());
                        }
                    }
                }
                if DAEMON_VERBOSE {
                    println!("Sending transactions to {}", socket_addr);
                }
                Ok(Message::Transactions { txs })
            }
            Message::MiniSketch { mini_sketch } => {
                if DAEMON_VERBOSE {
                    println!("Sending transactions request to {}", socket_addr);
                }

                let arena_r = arena_inner.read().unwrap();
                let socket_pk_read = *socket_pk_inner.read().unwrap();

                let perception = match arena_r.get_perception(&socket_pk_read) {
                    Some(some) => some,
                    None => return Err(DaemonError::Perceptionless.into()),
                };
                let peer_odd_sketch = arena_r
                    .get_status(&socket_pk_read)
                    .unwrap()
                    .get_odd_sketch();

                // Decode difference
                let perception_sketch = perception.get_mini_sketch();
                let (excess_actor_ids, missing_actor_ids) =
                    (perception_sketch - mini_sketch).decode().unwrap();
                let perception_odd_sketch = perception.get_odd_sketch();

                if DAEMON_VERBOSE {
                    println!(
                        "Decoding resulted in {} excess and {} missing",
                        excess_actor_ids.len(),
                        missing_actor_ids.len()
                    );
                }

                // Check for fraud
                if OddSketch::sketch_ids(&excess_actor_ids)
                    .xor(&OddSketch::sketch_ids(&missing_actor_ids))
                    == perception_odd_sketch.xor(&peer_odd_sketch)
                {
                    if DAEMON_VERBOSE {
                        println!("Valid Minisketch");
                    }
                    // Set expected IDs
                    rec_status_inner.write().unwrap().set_ids(&excess_actor_ids, &missing_actor_ids);
                    
                    Ok(Message::GetTransactions {
                        ids: missing_actor_ids,
                    })
                } else {
                    if DAEMON_VERBOSE {
                        println!("Fraudulent Minisketch");
                    }
                    // Stop reconciliation
                    rec_status_inner.write().unwrap().stop();
                    return Err(DaemonError::Perceptionless.into());
                }
            }
            Message::Reconcile => {
                if DAEMON_VERBOSE {
                    println!("Sending MiniSketch to {}", socket_addr);
                }

                // Add to reconcilee
                let socket_pk_read = *socket_pk_inner.read().unwrap();
                rec_status_inner.write().unwrap().add_reconcilee(&socket_pk_read);

                // Send the perceived minisketch
                let arena_r = arena_inner.read().unwrap();
                let socket_pk_read = *socket_pk_inner.read().unwrap();
                let perception = match arena_r.get_perception(&socket_pk_read) {
                    Some(some) => some,
                    None => return Err(DaemonError::Perceptionless.into()),
                };
                Ok(Message::MiniSketch {
                    mini_sketch: perception.get_mini_sketch(),
                })
            }
            _ => unreachable!(),
        });

        // Remove failed responses and merge with heartbeats
        let response_stream = response_stream.filter(|x| x.is_ok()).map(|x| x.unwrap());
        let out_stream = response_stream
            .select(odd_sketch_stream)
            .select(nonce_stream)
            .select(impulse_recv);

        // Send responses
        let send = sink.send_all(out_stream).map(|_| ()).or_else(|e| {
            println!("error = {:?}", e);
            Ok(())
        });
        tokio::spawn(send)
    });
    server
}
