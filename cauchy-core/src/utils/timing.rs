use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn get_current_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() * 1000 + u64::from(since_the_epoch.subsec_millis()) / 1_000_000
}

pub fn duration_from_millis(milli_secs: u64) -> Duration {
    Duration::from_millis(milli_secs)
}
