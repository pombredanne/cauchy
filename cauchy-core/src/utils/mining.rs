use bus::BusReader;
use bytes::Bytes;
use crossbeam::channel::Sender;

use crypto::sketches::odd_sketch::*;
use std::time;

use secp256k1::PublicKey;

use primitives::work_site::WorkSite;

pub fn mine(
    public_key: PublicKey,
    mut sketch_rx: BusReader<(OddSketch, Bytes)>,
    record_sender: Sender<(u64, u16)>,
    start_nonce: u64,
    step: u64,
) {
    println!("Start mining...");

    let mut best_nonce: u64 = 0;
    let mut best_distance: u16 = 512;

    let mut current_distance: u16;
    let (mut current_oddsketch, mut current_root) = sketch_rx.recv().unwrap();

    let work_site = WorkSite::new(public_key, current_root, start_nonce);

    loop {
        {
            match sketch_rx.try_recv() {
                Ok((new_oddsketch, new_root)) => {
                    current_oddsketch = new_oddsketch;
                    current_root = new_root;
                    current_distance = WorkSite::new(public_key, current_root, best_nonce)
                        .mine(&current_oddsketch);

                    record_sender.send((best_nonce, current_distance));
                    best_distance = current_distance;
                }
                Err(_) => {
                    current_distance = work_site.mine(&current_oddsketch);
                    if current_distance < best_distance {
                        best_nonce = work_site.get_nonce();
                        record_sender.send((best_nonce, current_distance));
                        best_distance = current_distance;
                    }
                    work_site.increment(step);
                }
            }
        }

        //thread::sleep(hash_interval); // TODO: Remove
    }
}
