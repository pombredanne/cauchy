use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use failure::Error;
use futures::{sync::mpsc, Future};
use log::{error, info, warn};
use tokio::codec::Framed;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

use crate::{
    crypto::sketches::{odd_sketch::*, *},
    db::{mongodb::MongoDB, storing::Storable},
    net::{heartbeats::*, messages::*},
    primitives::{
        arena::Arena,
        ego::{Ego, PeerEgo, Status, WorkState, WorkStatus},
        transaction::Transaction,
        tx_pool::TxPool,
    },
    utils::{
        constants::*,
        errors::{DaemonError, ImpulseReceiveError},
    },
};

macro_rules! daemon_info {
    ($($arg:tt)*) => {
        if config.debugging.daemon_verbose {
            info!(target: "daemon_event", $($arg)*);
        }
    };
}

macro_rules! daemon_warn {
    ($($arg:tt)*) => {
        if config.debugging.daemon_verbose {
            warn!(target: "daemon_event", $($arg)*);
        }
    };
}

macro_rules! daemon_error {
    ($($arg:tt)*) => {
        if config.debugging.daemon_verbose {
            error!(target: "daemon_event", $($arg)*);
        }
    };
}

pub enum Priority {
    Force,
    Standard,
}

pub enum Origin {
    Peer(Arc<Mutex<PeerEgo>>),
    RPC,
}

pub fn server(
    tx_db: MongoDB,
    ego: Arc<Mutex<Ego>>,
    socket_recv: mpsc::Receiver<TcpStream>,
    arena: Arc<Mutex<Arena>>,
    mempool: Arc<Mutex<TxPool>>,
    send_reconcile: mpsc::Sender<(Origin, TxPool, Priority)>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    daemon_info!("spawning deamon");

    // Bind socket
    let addr = format!("0.0.0.0:{}", config.network.server_port).to_string();
    let addr = addr.parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr)
        .map_err(|_| DaemonError::BindFailure)
        .unwrap();

    let incoming = listener
        .incoming()
        .map_err(|err| Error::from(DaemonError::SocketAcceptanceFailure { err }))
        .select(socket_recv.map_err(|_err| Error::from(DaemonError::Unreachable)))
        .map_err(|e| daemon_error!("error accepting socket; error = {:?}", e));

    let server = incoming.for_each(move |socket| {
        let socket_addr = socket.peer_addr().unwrap();
        daemon_info!("new server socket to {}", socket_addr);

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
        let send_reconcile_inner = send_reconcile.clone();
        let mempool_inner = mempool.clone();
        let response_stream = received_stream.filter_map(move |msg| match msg {
            Message::StartHandshake { secret } => {
                daemon_info!("received handshake initialisation from {}", socket_addr);

                Some(ego_inner.lock().unwrap().generate_end_handshake(secret))
            }
            Message::EndHandshake { pubkey, sig } => {
                daemon_info!("received handshake finalisation from {}", socket_addr);

                // If peer correctly signs our secret we upgrade them from a dummy pk
                arc_peer_ego.lock().unwrap().check_handshake(&sig, &pubkey);
                None
            }
            Message::Nonce { nonce } => {
                // Update nonce
                daemon_info!("received nonce from {}", socket_addr);

                arc_peer_ego.lock().unwrap().pull_nonce(nonce);
                None
            }
            Message::Work {
                oddsketch,
                root,
                nonce,
            } => {
                daemon_info!("received work from {}", socket_addr);

                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                // Update work
                if peer_ego_guard.get_status() == Status::Gossiping {
                    info!("pull work from {}", socket_addr);

                    peer_ego_guard.pull_work(oddsketch, nonce, root);
                    Some(Message::WorkAck)
                } else {
                    // TODO: Ban here
                    daemon_warn!("ignoring work from {}", socket_addr);

                    Some(Message::WorkNegAck)
                }
            }
            Message::MiniSketch { minisketch } => {
                info!("received minisketch from {}", socket_addr);

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

                    daemon_info!(
                        "minisketch decode reveals excess {} and mising {}",
                        excess_actor_ids.len(),
                        missing_actor_ids.len()
                    );

                    let perception_oddsketch = peer_ego_guard.get_perceived_oddsketch();
                    // Check for fraud
                    if OddSketch::sketch_ids(&excess_actor_ids)
                        .xor(&OddSketch::sketch_ids(&missing_actor_ids))
                        == perception_oddsketch.xor(&peer_oddsketch)
                    {
                        daemon_info!("minisketch passed validation");
                        // Set expected IDs
                        peer_ego_guard.update_ids(missing_actor_ids.clone());
                        peer_ego_guard.update_expected_minisketch(minisketch);

                        Some(Message::GetTransactions {
                            ids: missing_actor_ids,
                        })
                    } else {
                        daemon_error!("fraudulent minisketch from {}", socket_addr);
                        // TODO: Ban here
                        // Stop reconciliation
                        peer_ego_guard.update_status(Status::Gossiping);
                        None
                    }
                } else {
                    daemon_error!("received minisketch from non-pull target {}", socket_addr);
                    peer_ego_guard.update_status(Status::Gossiping);
                    None
                }
            }
            Message::GetTransactions { ids } => {
                // TODO: Check if reconcilee?
                daemon_info!("received transaction request from {}", socket_addr);

                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                // Init tx pool
                let mut tx_pool = TxPool::new(ids.len());

                // Find transactions
                for id in ids {
                    // if config.debugging.daemon_verbose {
                    //     println!("searching for transaction {:?}", id);
                    // }
                    match Transaction::from_db(&mut tx_db_inner.clone(), id.clone()) {
                        Ok(Some(tx)) => {
                            // if config.debugging.daemon_verbose {
                            //     println!("Found {:?}", id);
                            // }
                            tx_pool.insert(tx, Some(id.clone()), None);
                        }
                        Err(err) => {
                            daemon_error!("database error {:?}", err);
                            peer_ego_guard.update_status(Status::Gossiping);
                            return None;
                        }
                        Ok(None) => {
                            daemon_error!("transaction {:?} not found", id);
                            peer_ego_guard.update_status(Status::Gossiping);
                            return None;
                        }
                    }
                }
                let txs = tx_pool.to_sorted_txs();
                // Send transactions
                daemon_info!(
                    "replying to {} with {} transactions",
                    socket_addr,
                    txs.len()
                );
                peer_ego_guard.update_status(Status::Gossiping);
                Some(Message::Transactions { txs })
            }
            Message::Transactions { txs } => {
                daemon_info!("received transactions from {}", socket_addr);

                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                match peer_ego_guard.get_status() {
                    Status::StatePull => {
                        // Send to back stage
                        let mut tx_pool = TxPool::new(txs.len());
                        tx_pool.insert_batch(txs, true); // TODO: Catch out-of-order

                        tokio::spawn(
                            send_reconcile_inner
                                .clone()
                                .send((Origin::Peer(arc_peer_ego.clone()), tx_pool, Priority::Standard)) // TODO: Force vs Standard here
                                .map_err(|_| ())
                                .and_then(|_| future::ok(())),
                        );
                    },
                    Status::Gossiping => {
                        mempool_inner.lock().unwrap().insert_batch(txs, true);
                    },
                    Status::StatePush => {
                        // TODO: Ban

                    }
                }


                None
            }
            Message::Reconcile => {
                daemon_info!("received reconcile from {}", socket_addr);

                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                if peer_ego_guard.get_status() == Status::Gossiping {
                    daemon_info!("replying to {} with minisketch", socket_addr);

                    // Send minisketch
                    // Set status of peer push
                    peer_ego_guard.update_status(Status::StatePush);
                    Some(Message::MiniSketch {
                        minisketch: peer_ego_guard.get_perceived_minisketch(),
                    })
                } else {
                    daemon_info!("replying to {} with work negack", socket_addr);
                    return Some(Message::ReconcileNegAck);
                }
            }
            Message::WorkAck => {
                daemon_info!("received work ack from {}", socket_addr);

                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();
                peer_ego_guard.update_work_status(WorkStatus::Ready);
                peer_ego_guard.push_work();
                None
            }
            Message::WorkNegAck => {
                daemon_info!("received work ack from {}", socket_addr);

                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();
                peer_ego_guard.update_work_status(WorkStatus::Ready);
                None
            }
            Message::ReconcileNegAck => {
                daemon_info!("received reconcile negack from {}", socket_addr);
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();
                if peer_ego_guard.get_status() == Status::StatePull {
                    peer_ego_guard.update_status(Status::Gossiping);
                } else {
                    // TODO: Misbehaviour
                }
                None
            }
            Message::Peers { peers } => {
                unreachable!()
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
                daemon_error!("socket error {:?}", e);
                arena_inner.lock().unwrap().remove_peer(&socket_addr);
                Ok(())
            });
        tokio::spawn(send)
    });
    server
}
