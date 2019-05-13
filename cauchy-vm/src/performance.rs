use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use futures::future::{err, ok};
use futures::sink::Sink;
use futures::sync::mpsc::{Receiver, Sender};
use futures::sync::{mpsc, oneshot};
use futures::{Future, Stream};

use core::crypto::hashes::Identifiable;
use core::db::mongodb::MongoDB;
use core::db::storing::Storable;
use core::primitives::act::{Act, Message};
use core::primitives::transaction::Transaction;

use crate::vm::{Mailbox, VM};

pub struct Performance(pub HashMap<Bytes, Act>); // Actor ID: Total Act

impl Performance {
    pub fn append(&mut self, id: Bytes, act: Act) {
        if let Some(old_act) = self.0.get_mut(&id) {
            *old_act += act;
        } else {
            self.0.insert(id, act);
        }
    }
}

impl AddAssign for Performance {
    fn add_assign(&mut self, other: Performance) {
        for (key, act) in other.0 {
            match self.0.get(&key) {
                Some(other_act) => {
                    let mut new_act = act;
                    new_act += other_act.clone();
                    self.0.insert(key, new_act)
                }
                None => self.0.insert(key, act),
            };
        }
    }
}

impl Performance {
    pub fn new() -> Performance {
        Performance(HashMap::new())
    }

    pub fn add_read(&mut self, id: &Bytes, key: Bytes) {
        let act = match self.0.get_mut(id) {
            Some(some) => some,
            None => {
                self.0.insert(id.clone(), Act::new());
                self.0.get_mut(id).unwrap()
            }
        };
        act.access_pattern.read.insert(key);
    }

    pub fn add_write(&mut self, id: &Bytes, key: Bytes, value: Bytes) {
        let act = match self.0.get_mut(id) {
            Some(some) => some,
            None => {
                self.0.insert(id.clone(), Act::new());
                self.0.get_mut(id).unwrap()
            }
        };
        act.access_pattern.write.insert(key, value);
    }

    pub fn from_tx(
        tx_db: MongoDB,
        store: MongoDB,
        tx: Transaction,
    ) -> impl Future<Item = Performance, Error = ()> + Send {
        let (root_send, root_recv) = oneshot::channel();

        // Create new actor from tx binary
        let vm = VM::new(store.clone());

        // Create mail system
        let mut inboxes: HashMap<Bytes, Sender<Message>> = HashMap::new();
        let (outbox, outbox_recv) = mpsc::channel(512);

        let id = tx.get_id();
        let (first_mailbox, inbox_send) = Mailbox::new(outbox.clone());
        inboxes.insert(id.clone(), inbox_send);
        tokio::spawn(
            ok({
                vm.run(first_mailbox, tx, root_send).unwrap();
            })
            .and_then(move |_| {
                // For each new message
                outbox_recv.for_each(move |(message, parent_branch)| {
                    let receiver_id = message.get_receiver();
                    match inboxes.get(&receiver_id) {
                        // If receiver already live
                        Some(inbox_sender) => {
                            // Relay message to receiver
                            tokio::spawn(
                                inbox_sender
                                    .clone()
                                    .send(message)
                                    .map(|_| ())
                                    .map_err(|_| ()),
                            );
                            ok(())
                        }
                        // If receiver sleeping
                        None => {
                            // Load binary
                            let tx = match Transaction::from_db(tx_db.clone(), &receiver_id) {
                                Ok(Some(tx)) => tx,
                                Ok(None) => return err(()),
                                Err(_) => return err(()),
                            };
                            let id = tx.get_id();

                            // Boot receiver
                            let (new_mailbox, new_inbox_send) = Mailbox::new(outbox.clone());

                            // Add to list of live inboxes
                            inboxes.insert(tx.get_id(), new_inbox_send);

                            // Run receiver VM
                            tokio::spawn(ok({
                                vm.run(new_mailbox, tx, parent_branch).unwrap();
                                // Remove from live inboxes
                                inboxes.remove(&id);
                            }));
                            ok(())
                        }
                    }
                })
            }),
        );
        root_recv.map_err(|_| ())
    }
}
