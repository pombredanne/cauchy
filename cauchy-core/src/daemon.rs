use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use failure::Error;
use futures::Future;
use tokio::codec::Framed;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::sync::mpsc;

use crate::{
    crypto::sketches::{odd_sketch::*, *},
    db::{rocksdb::RocksDb, storing::Storable},
    net::{heartbeats::*, messages::*},
    primitives::{
        arena::Arena,
        ego::{Ego, PeerEgo, Status, WorkState, WorkStatus},
        transaction::Transaction,
    },
    utils::{
        constants::*,
        errors::{DaemonError, ImpulseReceiveError},
    },
};

pub fn server(
    tx_db: Arc<RocksDb>,
    ego: Arc<Mutex<Ego>>,
    socket_recv: mpsc::Receiver<TcpStream>,
    arena: Arc<Mutex<Arena>>,
    to_stage: mpsc::Sender<(Arc<Mutex<PeerEgo>>, HashSet<Transaction>)>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
        println!("spawning daemon");
    }

    // Bind socket
    let addr = format!("0.0.0.0:{}", CONFIG.NETWORK.SERVER_PORT).to_string();
    let addr = addr.parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr)
        .map_err(|_| DaemonError::BindFailure)
        .unwrap();

    let incoming = listener
        .incoming()
        .map_err(|err| Error::from(DaemonError::SocketAcceptanceFailure { err }))
        .select(socket_recv.map_err(|err| Error::from(DaemonError::Unreachable)))
        .map_err(|e| println!("error accepting socket; error = {:?}", e));

    let server = incoming.for_each(move |socket| {
        let socket_addr = socket.peer_addr().unwrap();
        if CONFIG.DEBUGGING.DAEMON_VERBOSE {
            println!("new server socket to {}", socket_addr);
        }

        // Construct peer ego
        let (peer_ego, peer_stream) = PeerEgo::new();

        // Send handshake
        peer_ego.send_msg(Message::StartHandshake {
            secret: peer_ego.get_secret(),
        });

        let arc_peer_ego = Arc::new(Mutex::new(peer_ego));
        let mut arena_guard = arena.lock().unwrap();
        arena_guard.new_peer(&socket_addr, arc_peer_ego.clone());
        drop(arena_guard);

        // Start work heartbeat
        let work_heartbeat = heartbeat_work(ego.clone(), arc_peer_ego.clone());

        // Frame the socket
        let framed_sock = Framed::new(socket, MessageCodec);
        let (send_stream, received_stream) = framed_sock.split();

        // Filter through received messages
        let tx_db_inner = tx_db.clone();
        let ego_inner = ego.clone();
        let to_stage_inner = to_stage.clone();
        let response_stream = received_stream.filter_map(move |msg| match msg {
            Message::StartHandshake { secret } => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received handshake initialisation from {}", socket_addr);
                    println!("replied with handshake finalisation from {}", socket_addr);
                }
                Some(ego_inner.lock().unwrap().generate_end_handshake(secret))
            }
            Message::EndHandshake { pubkey, sig } => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received handshake finalisation from {}", socket_addr);
                }

                // If peer correctly signs our secret we upgrade them from a dummy pk
                arc_peer_ego.lock().unwrap().check_handshake(&sig, &pubkey);
                None
            }
            Message::Nonce { nonce } => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received nonce from {}", socket_addr);
                }

                // Update nonce
                arc_peer_ego.lock().unwrap().pull_nonce(nonce);
                None
            }
            Message::Work {
                oddsketch,
                root,
                nonce,
            } => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received work from {}", socket_addr);
                }
                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                // Update work
                if peer_ego_guard.get_status() == Status::Gossiping {
                    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                        println!("pull work");
                    }
                    peer_ego_guard.pull_work(oddsketch, nonce, root);
                    Some(Message::WorkAck)
                } else {
                    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                        println!("ignore work");
                    }
                    Some(Message::WorkNegAck)
                }
            }
            Message::MiniSketch { minisketch } => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received minisketch from {}", socket_addr);
                }
                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                // Only respond if the pk is reconciliation target
                if peer_ego_guard.get_status() == Status::StatePull {
                    let peer_oddsketch = peer_ego_guard.get_oddsketch();

                    // Decode difference
                    let perception_minisketch = peer_ego_guard.get_perceived_minisketch();
                    let (excess_actor_ids, missing_actor_ids) = (perception_minisketch
                        - minisketch.clone())
                    .decode()
                    .unwrap();
                    let perception_oddsketch = peer_ego_guard.get_perceived_oddsketch();
                    println!(
                        "excess {}, mising {}",
                        excess_actor_ids.len(),
                        missing_actor_ids.len()
                    );

                    // Check for fraud
                    if OddSketch::sketch_ids(&excess_actor_ids)
                        .xor(&OddSketch::sketch_ids(&missing_actor_ids))
                        == perception_oddsketch.xor(&peer_oddsketch)
                    {
                        if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                            println!("valid minisketch");
                        }
                        // Set expected IDs
                        peer_ego_guard.update_ids(missing_actor_ids.clone());
                        peer_ego_guard.update_expected_minisketch(minisketch);

                        Some(Message::GetTransactions {
                            ids: missing_actor_ids,
                        })
                    } else {
                        if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                            println!("fraudulent minisketch");
                        }
                        // Stop reconciliation
                        peer_ego_guard.update_status(Status::Gossiping);
                        None
                    }
                } else {
                    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                        println!("received minisketch from non-pull target");
                    }
                    peer_ego_guard.update_status(Status::Gossiping);
                    None
                }
            }
            Message::GetTransactions { ids } => {
                // TODO: Check if reconcilee?
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received transaction request from {}", socket_addr);
                }

                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                // Find transactions
                let mut txs = HashSet::with_capacity(ids.len());
                for id in ids {
                    // if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    //     println!("searching for transaction {:?}", id);
                    // }
                    match Transaction::from_db(tx_db_inner.clone(), &id) {
                        Ok(Some(tx)) => {
                            // if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                            //     println!("Found {:?}", id);
                            // }
                            txs.insert(tx);
                        }
                        Err(err) => {
                            if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                                println!("database error {:?}", err);
                            }
                            peer_ego_guard.update_status(Status::Gossiping);
                            return None;
                        }
                        Ok(None) => {
                            if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                                println!("transaction {:?} not found", id);
                            }
                            peer_ego_guard.update_status(Status::Gossiping);
                            return None;
                        }
                    }
                }
                // Send transactions
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!(
                        "replying to {} with {} transactions",
                        socket_addr,
                        txs.len()
                    );
                }
                peer_ego_guard.update_status(Status::Gossiping);
                Some(Message::Transactions { txs })
            }
            Message::Transactions { txs } => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received transactions from {}", socket_addr);
                }

                // Add new txs to database
                // TODO: Fix danger here
                for tx in txs.iter() {
                    tx.to_db(tx_db_inner.clone());
                }

                // Send

                tokio::spawn(
                    to_stage_inner
                        .clone()
                        .send((arc_peer_ego.clone(), txs))
                        .map_err(|_| ())
                        .and_then(|_| future::ok(())),
                );
                None
            }
            Message::Reconcile => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received reconcile from {}", socket_addr);
                }
                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                if peer_ego_guard.get_status() == Status::Gossiping {
                    // Send minisketch
                    // Set status of peer push
                    peer_ego_guard.update_status(Status::StatePush);

                    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                        println!("replying with minisketch {}", socket_addr);
                    }

                    Some(Message::MiniSketch {
                        minisketch: peer_ego_guard.get_perceived_minisketch(),
                    })
                } else {
                    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                        println!("replying with negack {}", socket_addr);
                    }
                    return Some(Message::ReconcileNegAck);
                }
            }
            Message::WorkAck => {
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received work ack from {}", socket_addr);
                }
                peer_ego_guard.update_work_status(WorkStatus::Ready);
                peer_ego_guard.push_work();
                None
            }
            Message::WorkNegAck => {
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received work ack from {}", socket_addr);
                }
                peer_ego_guard.update_work_status(WorkStatus::Ready);
                None
            }
            Message::ReconcileNegAck => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received reconcile negack from {}", socket_addr);
                }
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();
                if peer_ego_guard.get_status() == Status::StatePull {
                    peer_ego_guard.update_status(Status::Gossiping);
                } else {
                    // TODO: Misbehaviour
                }
                None
            }
        });

        // Remove failed responses and merge with heartbeats
        let out_stream = response_stream
            .select(work_heartbeat)
            .select(peer_stream.map_err(|_| ImpulseReceiveError.into()));

        // Send responses
        let arena_inner = arena.clone();
        let send = send_stream
            .send_all(out_stream)
            .map(|_| ())
            .or_else(move |e| {
                println!("socket error {:?}", e);
                arena_inner.lock().unwrap().remove_peer(&socket_addr);
                Ok(())
            });
        tokio::spawn(send)
    });
    server
}
