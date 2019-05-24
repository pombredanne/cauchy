use std::sync::{Arc, Mutex};

use failure::Error;
use futures::future::ok;
use log::info;
use tokio::prelude::*;
use tokio::timer::Interval;

use crate::{
    ego::{ego::*, peer_ego::*, *},
    primitives::{arena::*, status::*},
    utils::{constants::*, errors::HeartBeatWorkError},
};

use super::messages::Message;

pub fn heartbeat_work(
    ego: Arc<Mutex<Ego>>,
    peer_ego: Arc<Mutex<PeerEgo>>,
) -> impl futures::stream::Stream<Item = Message, Error = Error> {
    Interval::new_interval(CONFIG.network.work_heartbeat_ms)
        .filter_map(move |_| {
            // Don't push work to anyone but gossipers
            let ego_guard = ego.lock().unwrap();
            let mut peer_ego_guard = peer_ego.lock().unwrap();
            if peer_ego_guard.get_status() != Status::Idle
                || peer_ego_guard.get_work_status() == WorkStatus::Idle
            {
                if CONFIG.debugging.heartbeat_verbose {
                    info!(target: "heartbeat_event",
                        "work heartbeat paused while {}",
                        peer_ego_guard.get_status().to_str()
                    )
                }
                None
            } else {
                // Send current work
                peer_ego_guard.update_work_status(WorkStatus::Idle);
                peer_ego_guard.commit_work(&ego_guard);
                Some(Message::Work {
                    oddsketch: ego_guard.get_oddsketch(),
                    root: ego_guard.get_root(),
                    nonce: ego_guard.get_nonce(),
                })
            }
        }) // Wait while reconciling or while sending to reconcilee
        .map_err(|_| HeartBeatWorkError.into())
}

pub fn heartbeat_reconcile(arena: Arc<Mutex<Arena>>) -> impl Future<Item = (), Error = ()> {
    Interval::new_interval(CONFIG.network.reconcile_heartbeat_ms)
        .map_err(|_| ()) // TODO: Catch
        .for_each(move |_| {
            arena.lock().unwrap().reconcile_leader();
            ok(())
        })
}
