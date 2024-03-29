use std::collections::{HashMap, HashSet};
use std::ops::AddAssign;
use std::sync::{Arc, Mutex};

use bus::Bus;
use bytes::{Bytes, BytesMut};
use failure::Error;
use futures::future::{err, ok};
use futures::sink::Sink;
use futures::sync::mpsc::{Receiver, Sender};
use futures::sync::{mpsc, oneshot};
use futures::{Future, Stream};
use log::info;

use crate::vm::performance::Performance;
use crate::vm::{Mailbox, VM};
use crate::{
    crypto::{
        hashes::Identifiable,
        sketches::{odd_sketch::OddSketch, SketchInsertable},
    },
    daemon::{Origin, Priority},
    db::{mongodb::*, storing::Storable},
    ego::ego::Ego,
    primitives::{
        act::{Act, Message},
        transaction::*,
        tx_pool::TxPool,
        work::WorkState,
    },
    utils::constants::{CONFIG, HASH_LEN},
};

pub struct Stage {
    ego: Arc<Mutex<Ego>>,
    db: MongoDB,
    ego_bus: Arc<Mutex<Bus<(OddSketch, Bytes)>>>,
}

impl Stage {
    pub fn new(ego: Arc<Mutex<Ego>>, db: MongoDB, ego_bus: Bus<(OddSketch, Bytes)>) -> Stage {
        Stage {
            ego,
            db,
            ego_bus: Arc::new(Mutex::new(ego_bus)),
        }
    }

    pub fn manager(
        self,
        mempool: Arc<Mutex<TxPool>>,
        incoming: futures::sync::mpsc::Receiver<(Origin, TxPool, Priority)>,
    ) -> impl Future<Item = (), Error = ()> + Send {
        incoming.for_each(move |(origin, txs, priority)| {
            let performances = match origin {
                Origin::Peer(peer_ego_arc) => {
                    unreachable!()
                    // self.process_txs_from_peer(peer_ego_arc.clone(), txs, priority)
                    // Update ego
                    // ego_guard.pull(
                    //     peer_ego_guard.get_oddsketch(),
                    //     peer_ego_guard.get_expected_minisketch(),
                    //     peer_ego_guard.get_root(),
                    // );
                }
                Origin::RPC => self.process_txs_from_rpc(txs.clone(), priority),
            };
            let done = futures::future::join_all(performances);
            done.wait();

            //Push to tx db and recreate ego
            let mut ego_guard = self.ego.lock().unwrap();
            let mut oddsketch = ego_guard.work_stack.get_oddsketch(); // TODO: Replace these with get &mut
            let mut minisketch = ego_guard.get_minisketch();

            for tx in txs.into_sorted_txs().iter() {
                tx.to_db(&mut self.db.clone(), None);
                oddsketch.insert(tx);
                minisketch.insert(tx);
            }
            let root = Bytes::from(&[0; HASH_LEN][..]); // TODO: Actually generate bytes
            let mut ego_bus_guard = self.ego_bus.lock().unwrap();
            ego_guard.work_stack.update_oddsketch(oddsketch.clone());
            ego_guard.update_minisketch(minisketch);
            ego_bus_guard.broadcast((oddsketch, root));

            ok(())
        })
    }

    pub fn process_txs_from_rpc(
        &self,
        txs: TxPool,
        priority: Priority,
    ) -> Vec<impl Future<Item = Performance, Error = ()> + Send> {
        info!(target: "stage_event", "processing tx batch from rpc");
        txs.into_sorted_txs()
            .iter()
            .map(|tx| Performance::from_tx(self.db.clone(), tx.clone()))
            .collect()
    }

    // pub fn process_txs_from_peer(
    //     &self,
    //     arc_peer_ego: Arc<Mutex<PeerEgo>>,
    //     &txs: HashSet<Transaction>,
    //     priority: Priority,
    // ) -> Vec<Future<Item = Performance, Error = ()> + Send> {
    //     // Lock ego and peer ego
    //     let mut ego_guard = self.ego.lock().unwrap();
    //     let mut peer_ego_guard = arc_peer_ego.lock().unwrap();

    //     // If received txs from reconciliation target check the payload matches reported
    //     if peer_ego_guard.get_status() == PeerStatus::StatePull {
    //         // Is reconcile target
    //         // Cease reconciliation status
    //         peer_ego_guard.update_status(PeerStatus::Gossiping);
    //         if peer_ego_guard.is_expected_payload(&txs) {
    //             // TODO: Send backstage and verify

    //             if config.debugging.stage_verbose {
    //                 println!("reconcile transactions sent to stage");
    //             }

    //             // TODO: Add to state function called here

    //             // self.ego_bus.br
    //             // Update ego
    //             // ego_guard.pull(
    //             //     peer_ego_guard.get_oddsketch(),
    //             //     peer_ego_guard.get_expected_minisketch(),
    //             //     peer_ego_guard.get_root(),
    //             // );
    //         }
    //     } else {
    //         if config.debugging.stage_verbose {
    //             println!("non-reconcile transactions sent to stage");
    //         }
    //     }

    //     // Send updated state immediately
    //     peer_ego_guard.update_status(PeerStatus::Gossiping);
    //     peer_ego_guard.update_work_status(WorkPeerStatus::Waiting);
    //     peer_ego_guard.commit_work(&ego_guard);

    //     tokio::spawn(
    //         peer_ego_guard
    //             .get_sink()
    //             .send(net::messages::Message::Work {
    //                 oddsketch: peer_ego_guard.get_oddsketch(),
    //                 root: peer_ego_guard.get_root(),
    //                 nonce: 0,
    //             })
    //             .map_err(|_| ())
    //             .and_then(|_| ok(())),
    //     );
    // }
}
