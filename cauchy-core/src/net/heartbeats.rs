use std::sync::{Arc, Mutex};

use failure::Error;
use futures::future::ok;
use tokio::prelude::*;
use tokio::timer::Interval;

use net::messages::Message;
use primitives::arena::*;
use primitives::ego::*;
use utils::constants::*;
use utils::errors::HeartBeatWorkError;
use utils::timing::*;

pub fn heartbeat_work(
    ego: Arc<Mutex<Ego>>,
    peer_ego: Arc<Mutex<PeerEgo>>,
) -> impl futures::stream::Stream<Item = Message, Error = Error> {
    Interval::new_interval(duration_from_millis(CONFIG.NETWORK.WORK_HEARTBEAT_MS))
        .filter_map(move |_| {
            let mut peer_ego_lock = peer_ego.lock().unwrap();
            let ego_lock = ego.lock().unwrap();
            if peer_ego_lock.get_status() != Status::Gossiping {
                if HEARTBEAT_VERBOSE {
                    println!("work heartbeat paused while {:?}", match peer_ego_lock.get_status() {
                        Status::StatePull => "pulling",
                        Status::StatePush => "pushing",
                        _ => unreachable!()
                    })
                }
                None
            } else {
                peer_ego_lock.push_work(&ego_lock);
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
        .for_each(move |_| {
            arena.lock().unwrap().reconcile_leader();
            ok(())
        })
}
