pub const TX_ID_LEN: usize = 64;
pub const PUBKEY_LEN: usize = 33;
pub const SIG_LEN: usize = 64;
pub const TX_DB_PATH: &str = ".geodesic/db/";
pub const IBLT_CHECKSUM_LEN: usize = 8;
pub const IBLT_PAYLOAD_LEN: usize = 64;
pub const SKETCH_CAPACITY: usize = 8;
pub const NONCE_HEARTBEAT_PERIOD_SEC: u64 = 100;
pub const NONCE_HEARTBEAT_PERIOD_NANO: u32 = 10_000_000;
pub const ODDSKETCH_HEARTBEAT_PERIOD_SEC: u64 = 100;
pub const ODDSKETCH_HEARTBEAT_PERIOD_NANO: u32 = 1_000_000;
pub const RECONCILE_HEARTBEAT_PERIOD_SEC: u64 = 1;
pub const RECONCILE_HEARTBEAT_PERIOD_NANO: u32 = 1_000_000;
