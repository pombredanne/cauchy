use net::messages::Message;
use primitives::arena::*;
use utils::constants::*;
use utils::timing::*;

use futures::future::ok;
use std::sync::{Arc, Mutex};
use tokio::prelude::*;
use tokio::timer::Interval;

use failure::Error;
use primitives::ego::*;
use utils::errors::HeartBeatWorkError;

pub fn heartbeat_work(
    ego: Arc<Mutex<Ego>>,
    peer_ego: Arc<Mutex<PeerEgo>>,
) -> impl futures::stream::Stream<Item = Message, Error = Error> {
    Interval::new_interval(duration_from_millis(CONFIG.NETWORK.WORK_HEARTBEAT_MS))
        .filter_map(move |_| {
            let mut peer_ego_lock = peer_ego.lock().unwrap();
            let ego_lock = ego.lock().unwrap();
            if peer_ego_lock.get_status() == Status::Gossiping {
                None
            } else {
                peer_ego_lock.witness(&ego_lock);
                Some(Message::Work {
                    oddsketch: ego_lock.get_oddsketch(),
                    root: ego_lock.get_root(),
                    nonce: ego_lock.get_nonce(),
                })
            }
        }) // Wait while reconciling or while sending to reconcilee
        .map_err(|_| HeartBeatWorkError.into())
}

// TODO: How does this thread die?
// TODO: Clean up
pub fn heartbeat_reconcile(arena: Arc<Mutex<Arena>>) -> impl Future<Item = (), Error = ()> {
    Interval::new_interval(duration_from_millis(CONFIG.NETWORK.RECONCILE_HEARTBEAT_MS))
        .map_err(|_| ()) // TODO: Catch
        .filter_map(move |_| {
            match arena.lock().unwrap().find_leader_sink() {
                None => None,
                Some(sink) => Some(tokio::spawn(
                    sink.send(Message::Reconcile)
                        .map_err(|e| ()) // TODO: Catch
                        .and_then(|_| ok(())),
                )),
            }
        })
        .for_each(|_| ok(()))
}
