use bytes::Bytes;
use std::convert::From;

#[derive(PartialEq, Debug)]
pub enum PassBy {
    Value,
    Reference,
}

#[derive(Debug)]
pub struct Script(PassBy, Bytes);

impl Script {
    pub fn new(passby: PassBy, raw: Bytes) -> Self {
        Script(passby, raw)
    }

    pub fn get_pass_by(&self) -> &PassBy {
        &self.0
    }
}

impl From<Script> for Bytes {
    fn from(item: Script) -> Self {
        match item {
            Script(_, some) => some
        }
    }
}

impl PartialEq for Script {
    fn eq (&self, other: &Script) -> bool {
        (self.0 == other.0) & (self.1 == other.1)
    }
}