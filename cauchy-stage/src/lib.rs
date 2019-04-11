use std::collections::{HashMap, HashSet};
use std::ops::AddAssign;
use std::sync::{Arc, Mutex};

use bus::Bus;
use bytes::{Bytes, BytesMut};
use core::primitives::transaction::*;
use failure::Error;
use futures::future::{err, ok};
use futures::sink::Sink;
use futures::sync::mpsc::{Receiver, Sender};
use futures::sync::{mpsc, oneshot};
use futures::{Future, Stream};

use core::{
    crypto::{hashes::Identifiable, sketches::odd_sketch::OddSketch},
    db::{rocksdb::*, storing::Storable},
    net,
    primitives::{
        act::{Act, Message},
        ego::{Ego, PeerEgo, Status, WorkState, WorkStatus},
    },
    utils::constants::CONFIG,
    daemon::{Origin, Priority}
};
use vm::vm::{Mailbox, VM};

pub struct Stage {
    ego: Arc<Mutex<Ego>>,
}

impl Stage {
    pub fn append_performance() {}

    pub fn manager(
        self,
        incoming: futures::sync::mpsc::Receiver<(Origin, HashSet<Transaction>, Priority)>,
        ego_bus: Bus<(OddSketch, Bytes)>,
    ) -> impl Future<Item = (), Error = ()> + Send {
        incoming.for_each(move |(origin, txs, priority)| {
            match origin {
                Origin::Peer(peer_ego_arc) => {
                    self.process_txs_from_peer(peer_ego_arc.clone(), txs, priority)
                }
                Origin::RPC => self.process_txs_from_rpc(txs, priority),
            }

            ok(())
        })
    }

    pub fn process_txs_from_rpc(&self, txs: HashSet<Transaction>, priority: Priority) {
        let mut ego_guard = self.ego.lock().unwrap();
    }

    pub fn process_txs_from_peer(
        &self,
        arc_peer_ego: Arc<Mutex<PeerEgo>>,
        txs: HashSet<Transaction>,
        priority: Priority,
    ) {
        // Lock ego and peer ego
        let mut ego_guard = self.ego.lock().unwrap();
        let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

        // If received txs from reconciliation target check the payload matches reported
        if peer_ego_guard.get_status() == Status::StatePull {
            // Is reconcile target
            // Cease reconciliation status
            peer_ego_guard.update_status(Status::Gossiping);
            if peer_ego_guard.is_expected_payload(&txs) {
                // TODO: Send backstage and verify

                if CONFIG.DEBUGGING.STAGE_VERBOSE {
                    println!("reconcile transactions sent to stage");
                }

                // Update ego
                // ego_guard.pull(
                //     peer_ego_guard.get_oddsketch(),
                //     peer_ego_guard.get_expected_minisketch(),
                //     peer_ego_guard.get_root(),
                // );
            }
        } else {
            if CONFIG.DEBUGGING.STAGE_VERBOSE {
                println!("non-reconcile transactions sent to stage");
            }
        }

        // Send updated state immediately
        peer_ego_guard.update_status(Status::Gossiping);
        peer_ego_guard.update_work_status(WorkStatus::Waiting);
        peer_ego_guard.commit_work(&ego_guard);

        tokio::spawn(
            peer_ego_guard
                .get_sink()
                .send(net::messages::Message::Work {
                    oddsketch: peer_ego_guard.get_oddsketch(),
                    root: peer_ego_guard.get_root(),
                    nonce: 0,
                })
                .map_err(|_| ())
                .and_then(|_| ok(())),
        );
    }
}

pub struct Performance {
    acts: HashMap<Bytes, Act>, // Actor ID: Total Act
}

impl Performance {
    pub fn append(&mut self, id: Bytes, act: Act) {
        if let Some(old_act) = self.acts.get_mut(&id) {
            *old_act += act;
        } else {
            self.acts.insert(id, act);
        }
    }
}

impl Performance {
    fn new(
        tx_db: Arc<RocksDb>,
        store: Arc<RocksDb>,
        tx: Transaction,
    ) -> impl Future<Item = Arc<Mutex<Performance>>, Error = ()> + Send {
        let performance = Arc::new(Mutex::new(Performance {
            acts: HashMap::new(),
        }));
        let (root_branch, _) = oneshot::channel();

        // Create new actor from tx binary
        let vm = VM::new(store.clone());

        // Create mail system
        let mut inboxes: HashMap<Bytes, Sender<Message>> = HashMap::new();
        let (outbox, outbox_recv) = mpsc::channel(512);

        let id = tx.get_id();
        let (first_mailbox, inbox_send) = Mailbox::new(outbox.clone());
        inboxes.insert(id.clone(), inbox_send);

        ok({
            let (first_act, result) = vm.run(first_mailbox, tx, root_branch);
            performance.clone().lock().unwrap().append(id, first_act);
        })
        .and_then(move |_| {
            // For each new message
            let performance_inner = performance.clone();
            let peformance_final = performance.clone();
            outbox_recv
                .for_each(move |(message, parent_branch)| {
                    // let performance_inner = performance_inner.clone();
                    let receiver_id = message.get_receiver();
                    match inboxes.get(&receiver_id) {
                        // If receiver already live
                        Some(inbox_sender) => {
                            // Relay message to receiver
                            tokio::spawn(
                                inbox_sender
                                    .clone()
                                    .send(message)
                                    .map(|_| ())
                                    .map_err(|_| ()),
                            );
                            ok(())
                        }
                        // If receiver sleeping
                        None => {
                            // Load binary
                            let tx = match Transaction::from_db(tx_db.clone(), &receiver_id) {
                                Ok(Some(tx)) => tx,
                                Ok(None) => return err(()),
                                Err(_) => return err(()),
                            };
                            let id = tx.get_id();

                            // Boot receiver
                            let (new_mailbox, new_inbox_send) = Mailbox::new(outbox.clone());

                            // Add to list of live inboxes
                            inboxes.insert(tx.get_id(), new_inbox_send);

                            // Run receiver VM
                            tokio::spawn(ok({
                                let (new_act, result) = vm.run(new_mailbox, tx, parent_branch);
                                performance_inner
                                    .lock()
                                    .unwrap()
                                    .append(id.clone(), new_act);

                                // Remove from live inboxes
                                inboxes.remove(&id);
                            }));
                            ok(())
                        }
                    }
                })
                .map(move |_| performance)
        })
    }
}
