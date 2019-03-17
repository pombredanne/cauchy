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
use primitives::status::Work;
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

        // Pair socket in connection manager
        let mut connection_manager_write = connection_manager.write().unwrap();
        let mut arena_write = arena.write().unwrap();
        let (impulse_recv, socket_pubkey) = connection_manager_write.add(&socket_addr).unwrap();
        let socket_pubkey_read = *socket_pubkey.read().unwrap();
        (*arena_write).new_peer(&socket_pubkey_read);

        // Send handshake
        connection_manager_write.send_handshake(&socket_addr);

        drop(arena_write);
        drop(connection_manager_write);

        // Frame the socket
        let framed_sock = Framed::new(socket, MessageCodec);
        let (send_stream, received_stream) = framed_sock.split();

        // Heartbeat OddSketch
        let update_stream = heartbeat_work(
            arena.clone(),
            local_status.clone(),
            rec_status.clone(),
            socket_pubkey.clone(),
            socket_addr,
        );

        // Filter through received messages
        let socket_pubkey_inner = socket_pubkey.clone();
        let arena_inner = arena.clone();
        let rec_status_inner = rec_status.clone();
        let local_status_inner = local_status.clone();
        let connection_manager_inner = connection_manager.clone();
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

                // If peer correctly signs our secret we upgrade them from a dummy pk
                let mut connection_manager_write = connection_manager_inner.write().unwrap();
                if connection_manager_write
                    .check_handshake(arena_inner.clone(), &socket_addr, sig, pubkey)
                    .unwrap()
                {
                    let arena_inner = arena_inner.clone();

                    let mut arena_write = arena_inner.write().unwrap();
                    let socket_pubkey_read = *socket_pubkey.read().unwrap();
                    (*arena_write).replace_key(&socket_pubkey_read, &pubkey);
                    drop(arena_write);
                    if DAEMON_VERBOSE {
                        println!("Handshake completed with {}", socket_addr);
                    }
                } else {
                    if DAEMON_VERBOSE {
                        println!("Handshake failed with {}", socket_addr);
                    }
                }
                // TODO: Reply with return StartHandshake?
                false
            }
            Message::Nonce { nonce } => {
                if DAEMON_VERBOSE {
                    println!("Received nonce from {}", socket_addr);
                }

                // Update nonce
                let socket_pubkey_read = *socket_pubkey.read().unwrap();
                let nonce = *nonce;
                command_peer!(arena_inner, socket_pubkey_read, update_nonce, nonce);
                false
            }
            Message::Work {
                oddsketch,
                root,
                nonce,
            } => {
                if DAEMON_VERBOSE {
                    println!("Received work from {}", socket_addr);
                }
                // Update state sketch
                let socket_pubkey_read = *socket_pubkey.read().unwrap();
                let new_work = Work {
                    oddsketch: oddsketch.clone(),
                    root: root.clone(),
                    nonce: *nonce,
                };
                command_peer!(arena_inner, socket_pubkey_read, update_work, new_work);
                false
            }
            Message::MiniSketch { .. } => {
                if DAEMON_VERBOSE {
                    println!("Received MiniSketch from {}", socket_addr);
                }

                // Only response if the pk is reconciliation target
                let socket_pubkey_read = *socket_pubkey.read().unwrap();
                rec_status_inner
                    .read()
                    .unwrap()
                    .target_eq(&socket_pubkey_read)
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
                let socket_pubkey_read = *socket_pubkey.read().unwrap();
                let rec_status_read = rec_status_inner.read().unwrap();
                if rec_status_read.target_eq(&socket_pubkey_read) {
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
                        let perception = match arena_r.get_perception(&socket_pubkey_read) {
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
                let socket_pubkey_read = *socket_pubkey_inner.read().unwrap();
                rec_status_inner
                    .write()
                    .unwrap()
                    .remove_reconcilee(&socket_pubkey_read);

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

                // TODO: Only do this if they are our reconicilee
                arena_inner
                    .read()
                    .unwrap()
                    .push_perception_to_peer(&socket_pubkey_read);

                Ok(Message::Transactions { txs })
            }
            Message::MiniSketch { minisketch } => {
                if DAEMON_VERBOSE {
                    println!("Sending transactions request to {}", socket_addr);
                }

                let arena_r = arena_inner.read().unwrap();
                let socket_pubkey_read = *socket_pubkey_inner.read().unwrap();

                let perception = match arena_r.get_perception(&socket_pubkey_read) {
                    Some(some) => some,
                    None => return Err(DaemonError::Perceptionless.into()),
                };
                let peer_oddsketch = arena_r
                    .get_status(&socket_pubkey_read)
                    .unwrap()
                    .get_oddsketch();

                // Decode difference
                let perception_sketch = perception.get_minisketch();
                let (excess_actor_ids, missing_actor_ids) =
                    (perception_sketch - minisketch).decode().unwrap();
                let perception_oddsketch = perception.get_oddsketch();

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
                    == perception_oddsketch.xor(&peer_oddsketch)
                {
                    if DAEMON_VERBOSE {
                        println!("Valid Minisketch");
                    }
                    // Set expected IDs
                    rec_status_inner
                        .write()
                        .unwrap()
                        .set_ids(&excess_actor_ids, &missing_actor_ids);

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
                let socket_pubkey_read = *socket_pubkey_inner.read().unwrap();
                rec_status_inner
                    .write()
                    .unwrap()
                    .add_reconcilee(&socket_pubkey_read);

                // Send the perceived minisketch
                let arena_r = arena_inner.read().unwrap();
                let socket_pubkey_read = *socket_pubkey_inner.read().unwrap();
                let perception = match arena_r.get_perception(&socket_pubkey_read) {
                    Some(some) => some,
                    None => return Err(DaemonError::Perceptionless.into()),
                };
                Ok(Message::MiniSketch {
                    minisketch: perception.get_minisketch(),
                })
            }
            _ => unreachable!(),
        });

        // Remove failed responses and merge with heartbeats
        let response_stream = response_stream.filter(|x| x.is_ok()).map(|x| x.unwrap());
        let out_stream = response_stream.select(update_stream).select(impulse_recv);

        // Send responses
        let connection_manager_inner = connection_manager.clone();
        let arena_inner = arena.clone();
        let rec_status_inner = rec_status.clone();
        let send = send_stream.send_all(out_stream).map(|_| ()).or_else(move |e| {
            println!("socket error {:?}", e);
            connection_manager_inner.write().unwrap().disconnect(arena_inner, rec_status_inner, &socket_addr);
            Ok(())
        });
        tokio::spawn(send)
    });
    server
}
