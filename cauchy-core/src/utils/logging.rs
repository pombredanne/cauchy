use chrono::Local;
use colored::*;
use log::{Level, Log, Metadata, Record};

use crate::utils::constants::CONFIG;

#[derive(Default)]
pub struct CLogger;

impl Log for CLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        // Check if the record is matched by the filter
        match record.target() {
            "arena_event" => {
                if !CONFIG.debugging.arena_verbose {
                    return;
                }
            }
            "heartbeat_event" => {
                if !CONFIG.debugging.heartbeat_verbose {
                    return;
                }
            }
            "daemon_event" => {
                if !CONFIG.debugging.daemon_verbose {
                    return;
                }
            }
            "encoding_event" => {
                if !CONFIG.debugging.encoding_verbose {
                    return;
                }
            }
            "decoding_event" => {
                if !CONFIG.debugging.decoding_verbose {
                    return;
                }
            }
            "stage_event" => {
                if !CONFIG.debugging.stage_verbose {
                    return;
                }
            }
            "mining_event" => {
                if !CONFIG.debugging.mining_verbose {
                    return;
                }
            }
            "stage_event" => {
                if !CONFIG.debugging.stage_verbose {
                    return;
                }
            }
            "rpc_event" => {
                if !CONFIG.debugging.rpc_verbose {
                    return;
                }
            }
            "ego_event" => {
                if !CONFIG.debugging.ego_verbose {
                    return;
                }
            }
            "vm_event" => {
                if !CONFIG.debugging.vm_verbose {
                    return;
                }
            }
            _ => (),
        }

        if self.enabled(record.metadata()) {
            let level_string = match record.level() {
                Level::Error => record.level().to_string().red(),
                Level::Warn => record.level().to_string().yellow(),
                Level::Info => record.level().to_string().cyan(),
                Level::Debug => record.level().to_string().purple(),
                Level::Trace => record.level().to_string().normal(),
            };
            println!(
                "{} {:<5} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S,%3f"),
                level_string,
                record.module_path().unwrap_or_default(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
