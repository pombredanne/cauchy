use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use failure::Error;
use futures::future::ok;
use log::info;
use tokio::prelude::*;
use tokio::timer::{Delay, Interval};

use crate::{
    primitives::arena::*,
    utils::{constants::*, errors::HeartBeatWorkError},
};

pub fn heartbeat(arena: Arc<Mutex<Arena>>) -> impl Future<Item = (), Error = ()> {
    Interval::new_interval(CONFIG.network.heartbeat_ms)
        .map_err(|_| ()) // TODO: Catch?
        .for_each(move |_| {
            arena.lock().unwrap().work_pulse(CONFIG.network.quorum_size);
            let when = Instant::now() + CONFIG.network.reconcile_timeout_ms;

            let arena_inner = arena.clone();
            let reconcile_task = Delay::new(when).map_err(|_| ()).and_then(move |_| {
                arena_inner.lock().unwrap().reconcile_leader();
                Ok(())
            });
            tokio::spawn(reconcile_task)
        })
}
