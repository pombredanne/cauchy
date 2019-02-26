pub const HASH_LEN: usize = 32;
pub const PUBKEY_LEN: usize = 33;
pub const SIG_LEN: usize = 64;
pub const TX_DB_PATH: &str = ".geodesic/db/";
pub const SKETCH_CAPACITY: usize = 32; // TODO: This should become dynamic
pub const NONCE_HEARTBEAT_PERIOD_SEC: u64 = 3;
pub const NONCE_HEARTBEAT_PERIOD_NANO: u32 = 0;
pub const ODDSKETCH_HEARTBEAT_PERIOD_SEC: u64 = 7;
pub const ODDSKETCH_HEARTBEAT_PERIOD_NANO: u32 = 0;
pub const RECONCILE_HEARTBEAT_PERIOD_SEC: u64 = 10;
pub const RECONCILE_HEARTBEAT_PERIOD_NANO: u32 = 0;
pub const SERVER_PORT: u16 = 8333;
pub const RPC_SERVER_PORT: u16 = 8332;
pub const MINER: bool = true;
pub const HEARTBEAT_VERBOSE: bool = true;
pub const DAEMON_VERBOSE: bool = true;
pub const PARSING_VERBOSE: bool = false;
pub const ARENA_VERBOSE: bool = false;
pub const ENCODING_VERBOSE: bool = false;
pub const DECODING_VERBOSE: bool = true;
