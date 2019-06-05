#[macro_use(bson, doc)]
use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use std::sync::{Arc, Mutex};

use bson::{spec::BinarySubtype, *};
use bytes::Bytes;
use futures::future::{err, lazy, ok};
use futures::sink::Sink;
use futures::sync::mpsc::{Receiver, Sender};
use futures::sync::{mpsc, oneshot};
use futures::{Future, Stream};
use log::info;
use stream_cancel::{StreamExt, Tripwire};
use tokio_threadpool::ThreadPool;

use crate::{
    crypto::hashes::Identifiable,
    db::{mongodb::MongoDB, storing::Storable, DataType, Database},
    primitives::{
        act::{Act, Message},
        transaction::Transaction,
    },
};

use super::{Mailbox, VM};

/* TODO: Given that each actor will write to one key
this probably best as some sort of concurrent hashmap */
#[derive(Clone, PartialEq, Eq, Default)]
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
    pub fn add_read(&mut self, id: &Bytes, key: Bytes) {
        let act = match self.0.get_mut(id) {
            Some(some) => some,
            None => {
                self.0.insert(id.clone(), Default::default());
                self.0.get_mut(id).unwrap()
            }
        };
        act.access_pattern.read.insert(key);
    }

    pub fn add_write(&mut self, id: &Bytes, key: Bytes, value: Bytes) {
        let act = match self.0.get_mut(id) {
            Some(some) => some,
            None => {
                self.0.insert(id.clone(), Default::default());
                self.0.get_mut(id).unwrap()
            }
        };
        act.access_pattern.write.insert(key, value);
    }

    pub fn from_tx(
        db: MongoDB,
        tx: Transaction,
    ) -> impl Future<Item = Performance, Error = ()> + Send {
        // Initialize performance
        let performance = Arc::new(Mutex::new(Performance::default()));

        let (root_send, root_recv) = oneshot::channel();

        // Create mail system
        info!(target: "vm_event", "initialising mail system");
        let mut inboxes: HashMap<Bytes, Sender<Message>> = HashMap::new();
        let (outbox, outbox_recv) = mpsc::channel(512);

        let id = tx.get_id(); // Used as the performance ID

        // Initialize mailboxes
        let (first_mailbox, inbox_send) = Mailbox::new(outbox.clone());

        // Add originating transaction to the mailbox
        inboxes.insert(id.clone(), inbox_send);

        let inboxes_inner = Arc::new(Mutex::new(inboxes));

        let performance_outer = performance.clone();
        let performance_inner = performance.clone();
        let vm_inner = VM::new(db.clone());
        let id_inner = id.clone();

        let pool = ThreadPool::new();

        pool.spawn(lazy(move || {
            info!(target: "vm_event", "spawning root vm");
            // Run
            ok({
                vm_inner
                    .run(first_mailbox, tx, id_inner, performance_inner, root_send)
                    .unwrap();
            })
        }));

        // For each new message
        info!(target: "vm_event", "watching outbox");
        let inboxes_inner = inboxes_inner.clone();
        let (trigger, tripwrire) = Tripwire::new();
        outbox_recv
            .take_until(tripwrire)
            .for_each(move |(message, parent_branch)| {
                let receiver_id = message.get_receiver();
                info!(target: "vm_event", "new message to {:?}", receiver_id);

                match inboxes_inner.lock().unwrap().get(&receiver_id) {
                    // If receiver already live
                    Some(inbox_sender) => {
                        info!("{:?} is live", receiver_id);
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
                        info!(target: "vm_event", "{:?} is not live", receiver_id);

                        // Load binary
                        let mut db = db.clone();
                        let tx = match Transaction::from_db(&mut db, receiver_id.clone()) {
                            Ok(Some(tx)) => tx,
                            Ok(None) => {
                                info!(target: "vm_event", "tx {:?} not found", receiver_id.clone());
                                return err(());
                            }
                            Err(_) => return err(()),
                        };

                        // Initialize receiver
                        info!(target: "vm_event", "spawning {:?} mailbox", receiver_id.clone());
                        let (new_mailbox, new_inbox_send) = Mailbox::new(outbox.clone());

                        // Add to list of live inboxes
                        let tx_id = tx.get_id();
                        inboxes_inner.lock().unwrap().insert(tx_id, new_inbox_send);

                        // Run receiver VM
                        let performance_inner = performance.clone();
                        let id_inner = id.clone();
                        let receiver_id_inner = receiver_id.clone();
                        let vm_inner = VM::new(db.clone());
                        let inboxes_inner = inboxes_inner.clone();
                        pool.spawn(lazy(move || {
                            info!(target: "vm_event", "spawning {:?} vm", receiver_id_inner);
                            vm_inner
                                .run(new_mailbox, tx, id_inner, performance_inner, parent_branch)
                                .unwrap();
                            // Remove from live inboxes
                            inboxes_inner.lock().unwrap().remove(&receiver_id);

                            ok(())
                        }));
                        ok(())
                    }
                }
            })
            .join(root_recv.map(|_| drop(trigger)).map_err(|_| ()))
            .map(move |_| match Arc::try_unwrap(performance_outer) {
                Ok(some) => {
                    info!(target: "vm_event", "performance complete");
                    some.into_inner().unwrap()
                }
                _ => unreachable!(),
            })
    }

    fn finalize(db: MongoDB, perfid: Bytes) {
        db.update(
            &DataType::State,
            doc! {"p" : Bson::Binary(BinarySubtype::Generic, perfid.to_vec())},
            doc! { "$unset" : {"p" : ""} },
        )
        .unwrap();
    }
}
