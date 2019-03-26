use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use failure::Error;
use futures::Future;
use tokio::codec::Framed;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::sync::mpsc;

use crypto::sketches::odd_sketch::*;
use crypto::sketches::*;
use db::rocksdb::RocksDb;
use db::storing::Storable;
use net::heartbeats::*;
use net::messages::*;
use primitives::arena::Arena;
use primitives::ego::{Ego, PeerEgo, Status, WorkState, WorkStatus};
use primitives::transaction::Transaction;
use utils::constants::*;
use utils::errors::{DaemonError, ImpulseReceiveError};

pub fn server(
    tx_db: Arc<RocksDb>,
    ego: Arc<Mutex<Ego>>,
    socket_recv: mpsc::Receiver<TcpStream>,
    arena: Arc<Mutex<Arena>>,
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
        let mut arena_locked = arena.lock().unwrap();
        arena_locked.new_peer(&socket_addr, arc_peer_ego.clone());
        drop(arena_locked);

        // Start work heartbeat
        let work_heartbeat = heartbeat_work(ego.clone(), arc_peer_ego.clone());

        // Frame the socket
        let framed_sock = Framed::new(socket, MessageCodec);
        let (send_stream, received_stream) = framed_sock.split();

        // Filter through received messages
        let tx_db_inner = tx_db.clone();
        let ego_inner = ego.clone();
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
                let mut peer_ego_locked = arc_peer_ego.lock().unwrap();

                // Update work
                if peer_ego_locked.get_status() == Status::Gossiping {
                    peer_ego_locked.pull_work(oddsketch, nonce, root);
                    Some(Message::WorkAck)
                } else {
                    None
                }
            }
            Message::MiniSketch { minisketch } => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received minisketch from {}", socket_addr);
                }
                // Lock peer ego
                let mut peer_ego_locked = arc_peer_ego.lock().unwrap();

                // Only respond if the pk is reconciliation target
                if peer_ego_locked.get_status() == Status::StatePull {
                    let peer_oddsketch = peer_ego_locked.get_oddsketch();

                    // Decode difference
                    let perception_minisketch = peer_ego_locked.get_perceived_minisketch();
                    let (excess_actor_ids, missing_actor_ids) = (perception_minisketch
                        - minisketch.clone())
                    .decode()
                    .unwrap();
                    let perception_oddsketch = peer_ego_locked.get_perceived_oddsketch();
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
                        peer_ego_locked.update_ids(missing_actor_ids.clone());
                        peer_ego_locked.update_expected_minisketch(minisketch);

                        Some(Message::GetTransactions {
                            ids: missing_actor_ids,
                        })
                    } else {
                        if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                            println!("fraudulent minisketch");
                        }
                        // Stop reconciliation
                        peer_ego_locked.update_status(Status::Gossiping);
                        None
                    }
                } else {
                    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                        println!("received minisketch from non-pull target");
                    }
                    peer_ego_locked.update_status(Status::Gossiping);
                    None
                }
            }
            Message::GetTransactions { ids } => {
                // TODO: Check if reconcilee?
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received transaction request from {}", socket_addr);
                }

                // Lock peer ego
                let mut peer_ego_locked = arc_peer_ego.lock().unwrap();

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
                            peer_ego_locked.update_status(Status::Gossiping);
                            return None;
                        }
                        Ok(None) => {
                            if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                                println!("transaction {:?} not found", id);
                            }
                            peer_ego_locked.update_status(Status::Gossiping);
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
                peer_ego_locked.update_status(Status::Gossiping);
                Some(Message::Transactions { txs })
            }
            Message::Transactions { txs } => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received transactions from {}", socket_addr);
                }
                // Lock ego and peer ego
                let mut ego_lock = ego_inner.lock().unwrap();
                let mut peer_ego_locked = arc_peer_ego.lock().unwrap();

                // If received txs from reconciliation target check the payload matches reported
                if peer_ego_locked.get_status() == Status::StatePull {
                    // Is reconcile target
                    // Cease reconciliation status
                    peer_ego_locked.update_status(Status::Gossiping);
                    if peer_ego_locked.is_expected_payload(&txs) {
                        // TODO: Send backstage and verify

                        // Add new txs to database
                        for tx in txs {
                            tx.to_db(tx_db_inner.clone());
                        }

                        // Update ego
                        ego_lock.pull(
                            peer_ego_locked.get_oddsketch(),
                            peer_ego_locked.get_expected_minisketch(),
                            peer_ego_locked.get_root(),
                        );
                        if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                            println!("reconciliation complete");
                        }
                    }
                } else {
                    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                        println!("non-pulled transactons");
                    }
                }

                // Send updated state immediately
                peer_ego_locked.update_status(Status::Gossiping);
                Some(Message::Work {
                    oddsketch: peer_ego_locked.get_oddsketch(),
                    root: peer_ego_locked.get_root(),
                    nonce: 0,
                })
            }
            Message::Reconcile => {
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received reconcile from {}", socket_addr);
                }
                // Lock peer ego
                let mut peer_ego_locked = arc_peer_ego.lock().unwrap();

                // If not gossiping then ignore reconcile request
                if peer_ego_locked.get_status() != Status::Gossiping {
                    if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                        println!("ignoring due to potential deadlock");
                    }
                    // Set to gossiping to avoid potential distributed deadlocks
                    peer_ego_locked.update_status(Status::Gossiping);
                    return None
                }

                // Set status of peer push
                peer_ego_locked.update_status(Status::StatePush);

                // Send minisketch
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("replying with minisketch {}", socket_addr);
                }
                Some(Message::MiniSketch {
                    minisketch: peer_ego_locked.get_perceived_minisketch(),
                })
            },
            Message::WorkAck => {
                let mut peer_ego_locked = arc_peer_ego.lock().unwrap();
                if CONFIG.DEBUGGING.DAEMON_VERBOSE {
                    println!("received work ack from {}", socket_addr);
                }
                peer_ego_locked.update_work_status(WorkStatus::Ready);
                peer_ego_locked.push_work();
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
