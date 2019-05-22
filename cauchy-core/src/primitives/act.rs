use std::ops::AddAssign;

use bytes::Bytes;

use super::access_pattern::*;

#[derive(Clone, PartialEq, Eq, Default)]
pub struct Act {
    pub access_pattern: AccessPattern,
    messages: Vec<Message>,
    operations: u64,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Message {
    sender: Bytes,
    receiver: Bytes,
    payload: Bytes,
}

impl Message {
    pub fn new(sender: Bytes, receiver: Bytes, payload: Bytes) -> Message {
        Message {
            sender,
            receiver,
            payload,
        }
    }

    pub fn get_receiver(&self) -> Bytes {
        self.receiver.clone()
    }

    pub fn get_sender(&self) -> Bytes {
        self.sender.clone()
    }

    pub fn get_payload(&self) -> Bytes {
        self.payload.clone()
    }
}

impl AddAssign for Act {
    fn add_assign(&mut self, other: Act) {
        self.access_pattern += other.access_pattern;
        self.messages.extend(other.messages);
        self.operations += other.operations;
    }
}
