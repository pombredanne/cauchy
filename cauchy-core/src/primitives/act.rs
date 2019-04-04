use std::ops::{Add, AddAssign};

use bytes::Bytes;

use super::access_pattern::*;

#[derive(Clone)]
pub struct Act {
    access_pattern: AccessPattern,
    messages: Vec<Message>,
    operations: u64,
}

impl Act {
    pub fn empty() -> Act {
        Act {
            access_pattern: AccessPattern::empty(),
            messages: Vec::new(),
            operations: 0,
        }
    }
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

    pub fn get_sender(&self) -> Bytes {
        self.sender.clone()
    }
}

impl Add for Act {
    type Output = Act;

    fn add(self, other: Act) -> Act {
        let mut self_msgs = self.messages;
        self_msgs.extend(other.messages);
        Act {
            access_pattern: self.access_pattern + other.access_pattern,
            messages: self_msgs,
            operations: self.operations + other.operations,
        }
    }
}

impl AddAssign for Act {
    fn add_assign(&mut self, other: Act) {
        self.access_pattern += other.access_pattern;
        self.messages.extend(other.messages);
        self.operations += other.operations;
    }
}
