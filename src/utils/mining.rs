use bus::BusReader;
use bytes::Bytes;
use primitives::work_site::WorkSite;
use secp256k1::PublicKey;
use std::sync::mpsc::Sender;
use std::time;

pub fn mine(
    public_key: PublicKey,
    start_nonce: u64,
    mut sketch_rx: BusReader<Bytes>,
    record_sender: Sender<(u64, u16)>,
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
                    work_site.increment();
                }
            }
        }

        //thread::sleep(hash_interval); // TODO: Remove
    }
}
