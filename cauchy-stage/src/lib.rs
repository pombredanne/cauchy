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
    daemon::{Origin, Priority},
    db::{rocksdb::*, storing::Storable},
    net,
    primitives::{
        act::{Act, Message},
        ego::{Ego, PeerEgo, Status, WorkState, WorkStatus},
    },
    utils::constants::CONFIG,
};
use vm::performance::Performance;
use vm::vm::{Mailbox, VM};

pub struct Stage {
    ego: Arc<Mutex<Ego>>,
    tx_db: Arc<RocksDb>,
    store: Arc<RocksDb>,
}

impl Stage {
    pub fn new(ego: Arc<Mutex<Ego>>, tx_db: Arc<RocksDb>, store: Arc<RocksDb>) -> Stage {
        Stage { ego, tx_db, store }
    }

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
        for tx in txs {
            Performance::from_tx(self.tx_db.clone(), self.store.clone(), tx);
        }

        // TODO: Add to state function called here
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

                // TODO: Add to state function called here

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
