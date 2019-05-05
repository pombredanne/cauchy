pub const HASH_LEN: usize = 32;
pub const VALUE_LEN: usize = 256;
pub const PUBKEY_LEN: usize = 33;
pub const SIG_LEN: usize = 64;
pub const TX_DB: &str = "tx_db";
pub const STORE_DB: &str = "store_db";
pub const SKETCH_CAPACITY: usize = 32; // TODO: This should become dynamic

use std::fs;
use std::io::Read;

use lazy_static::lazy_static;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Networking {
    pub WORK_HEARTBEAT_MS: u64,
    pub RECONCILE_HEARTBEAT_MS: u64,
    pub RECONCILE_TIMEOUT_MS: u64,
    pub SERVER_PORT: u16,
    pub RPC_SERVER_PORT: u16,
}

#[derive(Deserialize)]
pub struct Debugging {
    pub TEST_TX_INTERVAL: u64,
    pub ARENA_VERBOSE: bool,
    pub HEARTBEAT_VERBOSE: bool,
    pub DAEMON_VERBOSE: bool,
    pub ENCODING_VERBOSE: bool,
    pub DECODING_VERBOSE: bool,
    pub PARSING_VERBOSE: bool,
    pub STAGE_VERBOSE: bool,
}

#[derive(Deserialize)]
pub struct Mining {
    pub N_MINING_THREADS: u8,
}

#[derive(Deserialize)]
pub struct CoreConfig {
    pub NETWORK: Networking,
    pub MINING: Mining,
    pub DEBUGGING: Debugging,
}

lazy_static! {
    pub static ref CONFIG: CoreConfig = load_config();
}

pub fn default_config() -> CoreConfig {
    CoreConfig {
        NETWORK: Networking {
            WORK_HEARTBEAT_MS: 1_000,
            RECONCILE_HEARTBEAT_MS: 30_000,
            RECONCILE_TIMEOUT_MS: 5_000,
            SERVER_PORT: 8332,
            RPC_SERVER_PORT: 8333,
        },
        MINING: Mining {
            N_MINING_THREADS: 2,
        },
        DEBUGGING: Debugging {
            TEST_TX_INTERVAL: 500,
            ARENA_VERBOSE: false,
            HEARTBEAT_VERBOSE: false,
            DAEMON_VERBOSE: false,
            ENCODING_VERBOSE: false,
            DECODING_VERBOSE: false,
            PARSING_VERBOSE: false,
            STAGE_VERBOSE: true,
        },
    }
}

// TODO: Catch with defaults
pub fn load_config() -> CoreConfig {
    let mut path = dirs::home_dir().unwrap();
    path.push(".cauchy/config.toml");
    match &mut fs::File::open(path) {
        Ok(file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents);
            match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    println!("config file failed to parse {:?}", e);
                    println!("using default configuration");
                    default_config()
                }
            }
        }
        Err(e) => {
            println!("config file could not be read = {:?}", e);
            println!("using default configuration");
            default_config()
        }
    }
}
