use bytes::Bytes;
use std::convert::From;

#[derive(Debug, PartialEq, Clone)]
pub enum PassBy {
    Value,
    Reference,
}

#[derive(Debug, Clone, PartialEq)]
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