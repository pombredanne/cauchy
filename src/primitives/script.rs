use bytes::Bytes;
use std::convert::From;

pub const MAX_SCRIPT_LEN: usize = 1073741824; // One gigabyte

#[derive(Debug, Clone, PartialEq)]
pub struct Script(Bytes);

impl Script {
    pub fn new(raw: Bytes) -> Self {
        Script(raw)
    }
}

impl From<Script> for Bytes {
    fn from(item: Script) -> Self {
        match item {
            Script(some) => some
        }
    }
}