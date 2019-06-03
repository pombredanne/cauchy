use std::sync::mpsc::Sender;

use bus::BusReader;
use bytes::Bytes;
use log::info;
use secp256k1::PublicKey;

use crate::{
    crypto::sketches::{odd_sketch::OddSketch, SketchInsertable},
    primitives::work::WorkSite,
    utils::constants::{CONFIG, HASH_LEN},
};

pub fn mine(
    public_key: PublicKey,
    mut ego_recv: BusReader<(OddSketch, Bytes)>,
    record_sender: Sender<(u64, u16)>,
    start_nonce: u64,
) {
    info!(target: "mining_event", "mining thread started");

    let mut best_nonce: u64;
    let mut best_distance: u16 = 512;

    let mut current_distance: u16;

    // TODO: Load from disk here
    let mut current_oddsketch = Default::default();
    let mut current_root = Bytes::from(&[0; HASH_LEN][..]);

    let mut work_site = WorkSite::new(public_key, current_root, start_nonce);
    loop {
        {
            match ego_recv.try_recv() {
                Ok((new_oddsketch, new_root)) => {
                    info!(target: "mining_event", "mining reset");
                    current_oddsketch = new_oddsketch;
                    current_root = new_root;
                    work_site = WorkSite::new(public_key, current_root, start_nonce);
                    best_distance = 512;
                }
                Err(_) => {
                    current_distance = work_site.mine(current_oddsketch.clone());
                    if current_distance < best_distance {
                        info!(target: "mining_event", "new best found: {}", current_distance);
                        best_nonce = work_site.get_nonce();
                        record_sender.send((best_nonce, current_distance));
                        best_distance = current_distance;
                    }
                    work_site.increment();
                }
            }
        }
    }
}
