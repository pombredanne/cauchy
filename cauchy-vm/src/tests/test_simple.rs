
mod test_simple{
    use core::db::rocksdb::RocksDb;
    use core::db::storing::*;
    use core::db::*;
    use crate::vm::{VM};
    use bytes::Bytes;
    use std::sync::Arc;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_simple() {
        let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_syscall2/").unwrap();
        let mut file = File::open("src/tests/scripts/basic").unwrap();
        let mut script = Vec::new();
        file.read_to_end(&mut script).unwrap();

        let msg = Bytes::from(&b"Message"[..]);
        let vm_test = VM::new(Bytes::from(script), msg, 0, Arc::new(tx_db) );
        let result = vm_test.run();
    }
}