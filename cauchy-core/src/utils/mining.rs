use std::sync::mpsc::Sender;
use std::time;

use bus::BusReader;
use bytes::Bytes;
use secp256k1::PublicKey;

use crate::{
    crypto::sketches::{odd_sketch::OddSketch, SketchInsertable},
    primitives::work_site::WorkSite,
};

pub fn mine(
    public_key: PublicKey,
    mut ego_recv: BusReader<(OddSketch, Bytes)>,
    record_sender: Sender<(u64, u16)>,
    start_nonce: u64,
) {
    println!("Start mining...");

    let mut best_nonce: u64;
    let mut best_distance: u16 = 512;

    let mut current_distance: u16;

    // TODO: Load from disk here
    let mut current_oddsketch = OddSketch::new();
    let mut current_root = Bytes::new();

    let mut work_site = WorkSite::new(public_key, current_root, start_nonce);
    loop {
        {
            match ego_recv.try_recv() {
                Ok((new_oddsketch, new_root)) => {
                    current_oddsketch = new_oddsketch;
                    current_root = new_root;
                    work_site = WorkSite::new(public_key, current_root, start_nonce);
                    best_distance = 512;
                }
                Err(_) => {
                    current_distance = work_site.mine(&current_oddsketch);
                    if current_distance < best_distance {
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
