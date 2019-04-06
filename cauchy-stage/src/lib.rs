use std::collections::HashMap;
use std::sync::Arc;

use bytes::{Bytes, BytesMut};
use core::primitives::transaction::*;
use failure::Error;
use futures::future::{err, ok};
use futures::sink::Sink;
use futures::sync::mpsc::{Receiver, Sender};
use futures::sync::{mpsc, oneshot};
use futures::{Future, Stream};

use core::crypto::hashes::Identifiable;
use core::db::rocksdb::*;
use core::db::storing::Storable;
use core::primitives::act::{Act, Message};
use vm::vm::VM;

pub struct Stage {
    act: Act // This should not be entirely volatile
}

impl Stage {
    pub fn append_performance() {}
}

pub struct Performance {
    store: Arc<RocksDb>,
    tx_db: Arc<RocksDb>,
}

impl Performance {
    fn run(&self, tx: Transaction) -> impl Future<Item = Act, Error = ()> + Send + '_ {
        let (root_terminator, _) = oneshot::channel();

        // Create the outgoing message channel
        let (msg_send, msg_recv) = mpsc::channel::<(Message, oneshot::Sender<Act>)>(1337);

        // Create inbox channel holder
        let mut inboxes: HashMap<Bytes, Sender<Message>> = HashMap::new();

        // Create new actor from tx binary
        let (mut new_vm, _) = VM::new(
            tx.get_time(),
            tx.get_binary(),
            msg_send.clone(),
            root_terminator,
            self.store.clone(),
        );

        // Push aux data to inbox
        let (final_act_send, mut final_act) = oneshot::channel();
        msg_send
            .clone()
            .send((
                Message::new(Bytes::with_capacity(0), tx.get_id(), tx.get_aux()),
                final_act_send,
            ))
            .map_err(|_| ())
            .and_then(move |_| {
                // Boot first VM
                tokio::spawn(ok({
                    new_vm.run();
                    new_vm.terminate();
                }));

                // For each new message
                let router = msg_recv.for_each(move |(message, branch_terminator)| {
                    let sender_id = message.get_sender();
                    match inboxes.get(&sender_id) {
                        Some(inbox_sender) => {
                            tokio::spawn(
                                inbox_sender
                                    .clone()
                                    .send(message)
                                    .map(|_| ())
                                    .map_err(|_| ()),
                            );
                            ok(())
                        }
                        None => {
                            let tx = match Transaction::from_db(self.tx_db.clone(), &sender_id) {
                                Ok(Some(tx)) => tx,
                                Ok(None) => return err(()),
                                Err(_) => return err(()),
                            };
                            let (mut new_vm, new_inbox_sender) = VM::new(
                                tx.get_time(),
                                tx.get_binary(),
                                msg_send.clone(),
                                branch_terminator,
                                self.store.clone(),
                            );
                            inboxes.insert(tx.get_id(), new_inbox_sender);
                            tokio::spawn(ok({
                                new_vm.run();
                                new_vm.terminate();
                                inboxes.remove(&tx.get_id());
                            }));
                            ok(())
                        } // Spawn him
                    }
                });

                router
                    .and_then(move |_| final_act.try_recv().map(|opt| opt.unwrap()).map_err(|_| ()))
            })
    }
}
