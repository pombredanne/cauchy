use std::sync::Arc;
use std::thread;
use primitives::work_site::WorkSite;
use consensus::status::Status;
use std::time;

pub fn mine(work_site: Arc<WorkSite>, status: Arc<Status>) {
	println!("Starting mining...");

	let pk = work_site.get_public_key();

	let hash_interval = time::Duration::from_millis(10);

    let mut record_distance: u32;
    let mut current_distance: u32;

    let mut current_state_sketch = status.get_state_sketch();
    let mut best_nonce: u64;

    loop {
    	record_distance = WorkSite::new(pk, status.get_nonce()).mine(&current_state_sketch);
    	current_state_sketch  = status.get_state_sketch();
        current_distance = work_site.mine(&current_state_sketch);
        
        thread::sleep(hash_interval); // TODO: Remove
        println!("{}", record_distance);

        if current_distance < record_distance {
	        best_nonce = work_site.get_nonce();
            status.update_nonce(best_nonce);
        }

        work_site.increment();
    }
}