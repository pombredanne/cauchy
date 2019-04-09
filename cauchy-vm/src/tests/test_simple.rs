mod test_simple {
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

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
        // let mut file = File::open("src/tests/scripts/syscall").unwrap();
        let mut file = File::open("src/tests/scripts_rust/target/riscv64gc-unknown-none-elf/release/scripts_rust").unwrap();
        let mut script = Vec::new();
        file.read_to_end(&mut script).unwrap();

        let payload = Bytes::from(&b"TestFunc Message"[..]);
        let msg = Message::new(
            Bytes::from(&b"TestFunc Sender addr"[..]),
            Bytes::from(&b"TestFunc Receiver addr"[..]),
            payload,
        );
        tokio::run({
            // Create inbox

            // Dummy terminator for root
            let (parent_branch, _) = oneshot::channel();

            // Init the VM
            let vm = VM::new(Arc::new(store));

            // Construct session
            let (outbox, outbox_recv) = mpsc::channel(512);
            let tx = Transaction::new(407548800, Bytes::from(&b"aux"[..]), Bytes::from(script));
            let (mailbox, inbox_send) = Mailbox::new(outbox);

            inbox_send
                .clone()
                .send(msg)
                .map_err(|_| ())
                .map(|_| ()) // Send a msg to inbox
                .and_then(move |_| {
                    // Complete all spawned branches and print messages
                    tokio::spawn(
                        tokio::timer::Delay::new(Instant::now() + Duration::from_secs(1))
                            .map_err(|_| ())
                            .and_then(|_| {
                                outbox_recv.for_each(|(msg, parent_branch)| {
                                    parent_branch.send(()); // Complete branch
                                    println!(
                                        "{:?} received msg {:?} from {:?}",
                                        msg.get_receiver(),
                                        msg.get_payload(),
                                        msg.get_sender()
                                    );
                                    ok(())
                                })
                            }),
                    );
                    // Run the VM
                    ok({
                        println!("Execution start");
                        vm.run(mailbox, tx, parent_branch);
                        println!("Execution end");
                    })
                })
        });
    }

    // #[test]
    fn test_rust() {
        let store = RocksDb::open_db(".cauchy/tests/db_vm_test_rust/").unwrap();
        let mut file = File::open("src/tests/scripts_rust/target/riscv64gc-unknown-none-elf/release/scripts_rust").unwrap();
        let mut script = Vec::new();
        file.read_to_end(&mut script).unwrap();

        let payload = Bytes::from(&b"Message"[..]);
        let msg = Message::new(
            Bytes::from(&b"Sender addr"[..]),
            Bytes::from(&b"Receiver addr"[..]),
            payload,
        );
            // Dummy terminator for root
        let (parent_branch, _) = oneshot::channel();

        // Init the VM
        let vm = VM::new(Arc::new(store));

        // Construct session
        let (outbox, outbox_recv) = mpsc::channel(512);
        let tx = Transaction::new(407548800, Bytes::from(&b"aux"[..]), Bytes::from(script));
        let (mailbox, inbox_send) = Mailbox::new(outbox);

        let (_, result) = vm.run(mailbox, tx, parent_branch);
        assert_eq!(result.unwrap(), 8);

    }   
}
