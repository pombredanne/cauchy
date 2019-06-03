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
    ego::{ego::*, peer_ego::*},
    net::{heartbeats::*, messages::*},
    primitives::{
        arena::Arena,
        status::{PeerStatus, Status},
        transaction::Transaction,
        tx_pool::TxPool,
        work::WorkStack,
    },
    utils::{
        constants::*,
        errors::{DaemonError, ImpulseReceiveError},
    },
};

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
    info!(target: "daemon_event", "spawning deamon");

    // Bind socket
    let addr = format!("0.0.0.0:{}", CONFIG.network.server_port).to_string();
    let addr = addr.parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr)
        .map_err(|_| DaemonError::BindFailure)
        .unwrap();

    let incoming = listener
        .incoming()
        .map_err(|err| Error::from(DaemonError::SocketAcceptanceFailure { err }))
        .select(socket_recv.map_err(|_err| Error::from(DaemonError::Unreachable)))
        .map_err(|e| error!(target: "daemon_event", "error accepting socket; error = {:?}", e));

    let server = incoming.for_each(move |socket| {
        let socket_addr = socket.peer_addr().unwrap();
        info!(target: "daemon_event", "new server socket to {}", socket_addr);

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
                info!(target: "daemon_event", "received handshake initialisation from {}", socket_addr);

                Some(ego_inner.lock().unwrap().generate_end_handshake(secret))
            }
            Message::EndHandshake { pubkey, sig } => {
                info!(target: "daemon_event", "received handshake finalisation from {}", socket_addr);

                // If peer correctly signs our secret we upgrade them from a dummy pk
                arc_peer_ego.lock().unwrap().check_handshake(&sig, &pubkey);
                None
            }
            Message::Work(work_stack) => {
                info!(target: "daemon_event", "received work from {}", socket_addr);

                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();
                if peer_ego_guard.get_status() == PeerStatus::WorkPull {
                    // Update work
                    peer_ego_guard.update_status(PeerStatus::Fighting(work_stack));
                } else {
                    // TODO: Ban here
                    error!(target: "daemon_event", "received work from non-pull target")
                }

                None
            }
            Message::MiniSketch { minisketch } => {
                info!(target: "daemon_event", "received minisketch from {}", socket_addr);

                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                // TODO: This can be done properly
                let perceived_oddsketch = peer_ego_guard.get_perceived_oddsketch();
                let perceived_minisketch = peer_ego_guard.get_perceived_minisketch();

                // Only respond to reconciliation target
                match peer_ego_guard.get_status_mut() {
                    PeerStatus::StatePull(expectation) => {
                        match (perceived_minisketch, perceived_oddsketch) {
                            (Some(perceived_minisketch), Some(perceived_oddsketch)) => {
                                let peer_oddsketch = expectation.get_oddsketch();

                                // Decode difference
                                let (excess_actor_ids, missing_actor_ids) = (perceived_minisketch
                                    - minisketch.clone())
                                .decode()
                                .unwrap();

                                info!(
                                    target: "daemon_event", 
                                    "minisketch decode reveals excess {} and mising {}",
                                    excess_actor_ids.len(),
                                    missing_actor_ids.len()
                                );

                                // Check for fraud
                                if OddSketch::sketch_ids(&excess_actor_ids)
                                    .xor(&OddSketch::sketch_ids(&missing_actor_ids))
                                    == perceived_oddsketch.xor(&peer_oddsketch)
                                {
                                    info!(target: "daemon_event", "minisketch passed validation");

                                    // Set expected IDs
                                    expectation.update_ids(missing_actor_ids.clone());

                                    // Set expected minisketch
                                    expectation.update_minisketch(minisketch);

                                    Some(Message::GetTransactions {
                                        ids: missing_actor_ids,
                                    })
                                } else {
                                    error!(target: "daemon_event", "fraudulent minisketch from {}", socket_addr);
                                    // TODO: Ban here
                                    // Stop reconciliation
                                    peer_ego_guard.update_status(PeerStatus::Idle);
                                    None
                                }
                            }
                            _ => {
                                // TODO: Ban here
                                // TODO: More matches
                                error!(
                                    target: "daemon_event", 
                                    "received minisketch from {} while not pulling state",
                                    socket_addr
                                );
                                peer_ego_guard.update_status(PeerStatus::Idle);
                                None
                            }
                        }
                    }
                    _ => None, // TODO: Ban here
                }
            }
            Message::GetTransactions { ids } => {
                // TODO: Check if reconcilee?
                info!(target: "daemon_event", "received transaction request from {}", socket_addr);

                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                // Init tx pool
                let mut tx_pool = TxPool::with_capacity(ids.len());

                // Find transactions
                for id in ids {
                    // if CONFIG.debugging.daemon_verbose {
                    //     println!("searching for transaction {:?}", id);
                    // }
                    match Transaction::from_db(&mut tx_db_inner.clone(), id.clone()) {
                        Ok(Some(tx)) => {
                            // if CONFIG.debugging.daemon_verbose {
                            //     println!("Found {:?}", id);
                            // }
                            tx_pool.insert(tx, Some(id.clone()), None);
                        }
                        Err(err) => {
                            error!(target: "daemon_event", "database error {:?}", err);
                            peer_ego_guard.update_status(PeerStatus::Idle);
                            return None;
                        }
                        Ok(None) => {
                            error!(target: "daemon_event", "transaction {:?} not found", id);
                            peer_ego_guard.update_status(PeerStatus::Idle);
                            return None;
                        }
                    }
                }
                let txs = tx_pool.into_sorted_txs();
                // Send transactions
                info!(
                    target: "daemon_event", 
                    "replying to {} with {} transactions",
                    socket_addr,
                    txs.len()
                );
                peer_ego_guard.update_status(PeerStatus::Idle);
                Some(Message::Transactions { txs })
            }
            Message::Transactions { txs } => {
                info!(target: "daemon_event", "received transactions from {}", socket_addr);

                // Lock peer ego
                let peer_ego_guard = arc_peer_ego.lock().unwrap();

                match peer_ego_guard.get_status() {
                    PeerStatus::StatePull(expectation) => {
                        // Send to back stage
                        let mut tx_pool = TxPool::with_capacity(txs.len());
                        tx_pool.insert_batch(txs, true); // TODO: Catch out-of-order
                        drop(peer_ego_guard);

                        tokio::spawn(
                            send_reconcile_inner
                                .clone()
                                .send((
                                    Origin::Peer(arc_peer_ego.clone()),
                                    tx_pool,
                                    Priority::Standard,
                                )) // TODO: Force vs Standard here
                                .map_err(|_| ())
                                .and_then(|_| future::ok(())),
                        );

                        // Finished pull
                        ego_inner.lock().unwrap().update_status(Status::Idle);
                    }
                    PeerStatus::StatePush => {
                        // TODO: Ban here
                        error!(
                            target: "daemon_event", 
                            "received transactions from {} while pushing state",
                            socket_addr
                        );
                    }
                    _ => {
                        mempool_inner.lock().unwrap().insert_batch(txs, true);
                    }
                }

                None
            }
            Message::Reconcile => {
                info!(target: "daemon_event", "received reconcile from {}", socket_addr);

                // Lock peer ego
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

                if peer_ego_guard.get_status() == PeerStatus::Idle {
                    info!(target: "daemon_event", "replying to {} with minisketch", socket_addr);

                    // Send minisketch
                    // Set status of peer push
                    peer_ego_guard.update_status(PeerStatus::StatePush);
                    Some(Message::MiniSketch {
                        minisketch: peer_ego_guard.get_perceived_minisketch()?,
                    })
                } else {
                    info!(target: "daemon_event", "replying to {} with work negack", socket_addr);
                    Some(Message::ReconcileNegAck)
                }
            }
            Message::GetWork => {
                info!(target: "daemon_event", "received get work from {}", socket_addr);
                let ego_guard = ego_inner.lock().unwrap();
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();
                let work_stack = ego_guard.get_work_stack();
                peer_ego_guard.push_work(work_stack, ego_guard.get_minisketch());
                None
            }
            Message::ReconcileNegAck => {
                info!(target: "daemon_event", "received reconcile negack from {}", socket_addr);
                let mut peer_ego_guard = arc_peer_ego.lock().unwrap();
                match peer_ego_guard.get_status() {
                    PeerStatus::StatePull(_) => peer_ego_guard.update_status(PeerStatus::Idle),
                    _ => {
                        // TODO: Misbehaviour
                        error!(
                            target: "daemon_event", 
                            "received negack from {} while not pulling state",
                            socket_addr
                        );
                    }
                };
                None
            }
            Message::Peers { peers } => unreachable!(),
        });

        // Remove failed responses and merge with heartbeats
        let out_stream =
            response_stream.select(peer_stream.map_err(|_| ImpulseReceiveError.into()));

        // Send responses
        let arena_inner = arena.clone();
        let send = send_stream
            .send_all(out_stream)
            .map(|_| ())
            .or_else(move |e| {
                error!(target: "daemon_event", "socket error {:?}", e);
                arena_inner.lock().unwrap().remove_peer(&socket_addr);
                Ok(())
            });
        tokio::spawn(send)
    });
    server
}
