pub const HASH_LEN: usize = 32;
pub const PUBKEY_LEN: usize = 33;
pub const SIG_LEN: usize = 64;
pub const TX_DB_PATH: &str = ".geodesic/db/";
pub const SKETCH_CAPACITY: usize = 32; // TODO: This should become dynamic
pub const HEARTBEAT_VERBOSE: bool = true;
pub const DAEMON_VERBOSE: bool = true;
pub const PARSING_VERBOSE: bool = false;
pub const ARENA_VERBOSE: bool = false;
pub const ENCODING_VERBOSE: bool = false;
pub const DECODING_VERBOSE: bool = true;

use std::fs;
use std::io::Read;

#[derive(Deserialize)]
pub struct Networking {
    pub WORK_HEARTBEAT: u64,
    pub RECONCILE_HEARTBEAT: u64,
    pub RECONCILE_TIMEOUT: u64,
    pub SERVER_PORT: u16,
    pub RPC_SERVER_PORT: u16,
}

#[derive(Deserialize)]
pub struct Debugging {
    pub TEST_TX_INTERVAL: u64,
}

#[derive(Deserialize)]
pub struct Mining {
    pub N_MINING_THREADS: u8,
}

#[derive(Deserialize)]
pub struct CoreConfig {
    pub NETWORK: Networking,
    pub MINING: Mining,
    pub DEBUGGING: Debugging
}

lazy_static! {
    pub static ref CONFIG: CoreConfig = load_config();
}

pub fn default_config() -> CoreConfig {
    CoreConfig {
        NETWORK: Networking {
            WORK_HEARTBEAT: 1_000_000_000,
            RECONCILE_HEARTBEAT: 3_000_000_000,
            RECONCILE_TIMEOUT: 5_000_000_000,
            SERVER_PORT: 8332,
            RPC_SERVER_PORT: 8333
        },
        MINING: Mining {
            N_MINING_THREADS: 2
        },
        DEBUGGING: Debugging {
            TEST_TX_INTERVAL: 500_000
        }
    }
}

// TODO: Catch with defaults
pub fn load_config() -> CoreConfig {
    let mut path = dirs::home_dir().unwrap();
    path.push(".geodesic/config.toml");
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
        },
        Err(e) => {
            println!("config file could not be read = {:?}", e);
            println!("using default configuration");
            default_config()
        }
    } 
}