mod test_simple {
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;

    use bytes::Bytes;
    use futures::future::{ok, Future};
    use futures::sink::Sink;
    use futures::stream::Stream;
    use futures::sync::{mpsc, oneshot};

    use core::db::rocksdb::RocksDb;
    use core::db::*;
    use core::primitives::act::Message;
    use core::primitives::transaction::Transaction;

    use crate::vm::{Mailbox, VM};

    #[test]
    fn test_simple() {
        let store = RocksDb::open_db(".cauchy/tests/db_vm_test_simple/").unwrap();
        let mut file = File::open("src/tests/scripts/syscall").unwrap();
        let mut script = Vec::new();
        file.read_to_end(&mut script).unwrap();

        let payload = Bytes::from(&b"Message"[..]);
        let msg = Message::new(
            Bytes::from(&b"Sender addr"[..]),
            Bytes::from(&b"Receiver addr"[..]),
            payload,
        );
        tokio::run({
            // Create inbox

            // Dummy terminator for root
            let (parent_branch, _) = oneshot::channel();

            // Init the VM
            let mut vm = VM::new(Arc::new(store));

            // Construct session
            let tx = Transaction::new(407548800, Bytes::from(&b"aux"[..]), Bytes::from(script));
            let (mailbox, inbox_send, outbox_recv) = Mailbox::new();


            // Session


            inbox_send
                .clone()
                .send(msg)
                .map_err(|_| ())
                .map(|_| ()) // Send a msg to inbox
                .and_then(move |_| {
                    ok({
                        vm.run(mailbox, tx, parent_branch);
                    })
                }) // Run the VM
                .and_then(|_| {
                    outbox_recv.for_each(|(msg, _)| {
                        println!(
                            "Received output msg {:?} sent to {:?}",
                            msg.get_payload(),
                            msg.get_receiver()
                        );
                        ok(())
                    })
                }) // Print the outgoing msgs
        });
        // assert!(false);
    }
}
