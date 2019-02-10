use bus::BusReader;
use bytes::Bytes;
use crossbeam::channel::Sender;

use std::time;

use secp256k1::PublicKey;

use primitives::work_site::WorkSite;

pub fn mine(
    public_key: PublicKey,
    mut sketch_rx: BusReader<Bytes>,
    record_sender: Sender<(u64, u16)>,
    start_nonce: u64,
    step: u64,
) {
    let work_site = WorkSite::new(public_key, start_nonce);
    println!("Start mining...");

    let mut best_nonce: u64 = 0;
    let mut best_distance: u16 = 512;

    let pk = work_site.get_public_key();

    let mut current_distance: u16;
    let mut current_sketch: Bytes = sketch_rx.recv().unwrap();

    loop {
        {
            match sketch_rx.try_recv() {
                Ok(sketch) => {
                    current_sketch = sketch;
                    current_distance = WorkSite::new(pk, best_nonce).mine(&current_sketch);

                    record_sender.send((best_nonce, current_distance));
                    best_distance = current_distance;
                }
                Err(_) => {
                    current_distance = work_site.mine(&current_sketch);
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
