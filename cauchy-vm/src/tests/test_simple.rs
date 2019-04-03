mod test_simple {
    use crate::vm::VM;
    use bytes::Bytes;
    use core::db::rocksdb::RocksDb;
    use core::db::storing::*;
    use core::db::*;
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;
    use futures::sync::{oneshot, mpsc};
    use futures::sync::mpsc::{Sender, Receiver};

    #[test]
    fn test_simple() {
        let store = RocksDb::open_db(".cauchy/tests/db_vm_test_simple/").unwrap();
        let mut file = File::open("src/tests/scripts/basic").unwrap();
        let mut script = Vec::new();
        file.read_to_end(&mut script).unwrap();

        let msg = Bytes::from(&b"Message"[..]);
        let (msg_sender, msg_recv) = mpsc::channel(1337); // TODO: We can use an unbounded channel if we'd like here? Or perhaps this is part of the limit
        let (mut vm_test, inbox_send) = VM::new(0, Bytes::from(script), msg_sender, Arc::new(store));

        let result = vm_test.run();
        assert!(result.is_ok());
    }
}
