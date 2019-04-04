mod test_simple {
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;

    use bytes::Bytes;
    use futures::future::{ok, Future};
    use futures::sink::Sink;
    use futures::sync::{mpsc, oneshot};

    use core::db::rocksdb::RocksDb;
    use core::db::*;
    use core::primitives::act::Message;

    use crate::vm::VM;

    #[test]
    fn test_simple() {
        let store = RocksDb::open_db(".cauchy/tests/db_vm_test_simple/").unwrap();
        let mut file = File::open("src/tests/scripts/basic").unwrap();
        let mut script = Vec::new();
        file.read_to_end(&mut script).unwrap();

        let payload = Bytes::from(&b"Message"[..]);
        let msg = Message::new(Bytes::from(&b"Sender addr"[..]), Bytes::from(&b"Receiver addr"[..]), payload);
        tokio::run(
            {
                // Create inbox
                let (msg_sender, _) = mpsc::channel(128);
                
                // Dummy terminator for root
                let (root_terminator, _) = oneshot::channel();
                
                // Create the VM
                let (mut vm_test, inbox_sender) = VM::new(0, Bytes::from(script), msg_sender, root_terminator, Arc::new(store));

                // Send a message to it
                tokio::spawn(inbox_sender.clone().send(msg).map_err(|_| ()).map(|_| ()));

                // Run it
                let result = vm_test.run();
                assert!(result.is_ok());

                ok(())
            }
        )
    }
}
