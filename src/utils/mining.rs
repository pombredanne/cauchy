use bytes::Bytes;
use std::sync::Arc;
use std::thread;
use primitives::work_site::WorkSite;
use consensus::status::Status;
use std::time;
use std::time::SystemTime;


pub fn mine(work_site: Arc<WorkSite>, status: Arc<Status>) {
	println!("Starting mining...");

	let ten_millis = time::Duration::from_millis(10);


    let mut now = SystemTime::now();
    let mut i = 0;

    let mut record_distance: u32 = 512;
    let mut current_distance: u32;

    let mut current_state_sketch: Bytes;

    loop {
    	current_state_sketch  = status.get_state_sketch();
        current_distance = work_site.mine(&current_state_sketch);
        thread::sleep(ten_millis); // TODO: Remove

        if current_distance < record_distance {
	        record_distance = current_distance;

            status.update_state_sketch(current_state_sketch);
            status.update_nonce(work_site.get_nonce());

            println!("\nNew best found!");
            println!(
                "{} seconds since last discovery",
                now.elapsed().unwrap().as_secs()
            );
            println!("{} hashes since last discovery", i);
            println!("Distance to state {}", record_distance);
            i = 0;
            now = SystemTime::now();
        }

        work_site.increment();
        i += 1;
    }
}