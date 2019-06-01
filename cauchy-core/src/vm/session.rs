use std::sync::{Arc, Mutex};

use bytes::Bytes;
use futures::{
    future::{ok, Future},
    sink::Sink,
    stream::Stream,
    sync::oneshot,
    Async,
};

use super::performance::Performance;
use crate::{
    db::mongodb::MongoDB,
    vm::{Mailbox, Message},
};

pub struct ValueStore(Bytes);

pub struct Session {
    pub mailbox: Mailbox,
    pub id: Bytes,
    pub perfid: Bytes,
    pub timestamp: u64,
    pub binary_hash: Bytes,
    pub aux: Bytes,
    pub performance: Arc<Mutex<Performance>>,
    pub child_branch: Option<oneshot::Receiver<()>>,
    pub store: MongoDB,
}

impl Session {
    pub fn recv(&mut self) -> Option<Message> {
        // Wait while children still live
        if let Some(branch) = self.child_branch.take() {
            let child_perforamnce = branch.wait().unwrap();
        }

        match self.mailbox.inbox.poll() {
            Ok(Async::Ready(msg)) => msg,
            _ => unreachable!(),
        }
    }

    pub fn send(&mut self, msg: Message) {
        // Wait whiel children still live
        if let Some(branch) = self.child_branch.take() {
            let child_perforamnce = branch.wait().unwrap();
        }

        let (child_send, child_branch) = oneshot::channel();
        self.child_branch = Some(child_branch);
        tokio::spawn(
            self.mailbox
                .outbox
                .clone()
                .send((msg, child_send))
                .map_err(|_| ())
                .and_then(|_| ok(())),
        );
    }
}
