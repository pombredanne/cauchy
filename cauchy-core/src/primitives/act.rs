use bytes::Bytes;
use std::collections::HashSet;

use super::access_pattern::*;

pub struct Act {
    access_pattern: AccessPattern,
    messages: HashSet<Message>,
    operations: u64
}

impl Act {
    pub fn empty() -> Act {
        Act {
            access_pattern: AccessPattern::empty(),
            messages: HashSet::with_capacity(0),
            operations: 0
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Message {
    sender: Bytes,
    receiver: Bytes,
    payload: Bytes
}

impl Message {
    pub fn new(sender: Bytes, receiver: Bytes, payload: Bytes) -> Message {
        Message {
            sender,
            receiver,
            payload
        }
    } 

    pub fn get_sender(&self) -> Bytes {
        self.sender.clone()
    }
}