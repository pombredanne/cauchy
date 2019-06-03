pub const HASH_LEN: usize = 32;
pub const VALUE_LEN: usize = 256;
pub const PUBKEY_LEN: usize = 33;
pub const SIG_LEN: usize = 64;
pub const SKETCH_CAPACITY: usize = 32; // TODO: This should become dynamic

use std::fs;
use std::io::Read;
use std::time::Duration;

use lazy_static::lazy_static;
use log::warn;
use serde::{Deserialize, Deserializer};
use serde_derive::Deserialize;

use super::timing::duration_from_millis;

#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Networking {
    #[serde(deserialize_with = "from_u64")]
    pub heartbeat_ms: Duration,
    pub quorum_size: usize,
    #[serde(deserialize_with = "from_u64")]
    pub reconcile_timeout_ms: Duration,
    pub server_port: u16,
    pub rpc_server_port: u16,
}

impl Default for Networking {
    fn default() -> Self {
        Networking {
            heartbeat_ms: duration_from_millis(1_000),
            quorum_size: 32,
            reconcile_timeout_ms: duration_from_millis(5_00),
            server_port: 8332,
            rpc_server_port: 8333,
        }
    }
}

fn from_u64<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let u: u64 = Deserialize::deserialize(deserializer)?;
    Ok(duration_from_millis(u))
}

#[derive(Deserialize)]
pub struct Debugging {
    #[serde(deserialize_with = "from_u64")]
    pub test_tx_interval: Duration,
    pub arena_verbose: bool,
    pub heartbeat_verbose: bool,
    pub daemon_verbose: bool,
    pub encoding_verbose: bool,
    pub decoding_verbose: bool,
    pub parsing_verbose: bool,
    pub stage_verbose: bool,
    pub rpc_verbose: bool,
    pub mining_verbose: bool,
    pub ego_verbose: bool,
    pub vm_verbose: bool,
}

impl Default for Debugging {
    fn default() -> Self {
        Debugging {
            test_tx_interval: duration_from_millis(500), // TODO: Remove?
            arena_verbose: true,
            heartbeat_verbose: true,
            daemon_verbose: true,
            encoding_verbose: false,
            decoding_verbose: false,
            parsing_verbose: true,
            stage_verbose: true,
            rpc_verbose: true,
            mining_verbose: true,
            ego_verbose: true,
            vm_verbose: true,
        }
    }
}

#[derive(Deserialize)]
pub struct Mining {
    pub n_mining_threads: u8,
}

impl Default for Mining {
    fn default() -> Self {
        Mining {
            n_mining_threads: 2,
        }
    }
}

#[derive(Deserialize, Default)]
pub struct Config {
    pub network: Networking,
    pub mining: Mining,
    pub debugging: Debugging,
}

lazy_static! {
    pub static ref CONFIG: Config = load_config();
}

pub fn load_config() -> Config {
    let mut path = dirs::home_dir().unwrap();
    path.push(".cauchy/config.toml");
    match &mut fs::File::open(path) {
        Ok(file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents);
            match toml::from_str(&contents) {
                Ok(r_config) => r_config,
                Err(e) => {
                    warn!(target: "startup_event", "config file failed to parse {:?}, using default configuration", e);
                    Default::default()
                }
            }
        }
        Err(e) => {
            warn!(target: "startup_event", "config file could not be read = {:?}, using default configuration", e);
            Default::default()
        }
    }
}
