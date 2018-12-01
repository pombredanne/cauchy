use bytes::Bytes;
use std::convert::From;

pub struct Script(Bytes);

impl From<Bytes> for Script {
    fn from(item: Bytes) -> Self {
        Script(item)
    }
}

impl From<Script> for Bytes {
    fn from(item: Script) -> Self {
        match item {
            Script(v) => v
        }
    }
}