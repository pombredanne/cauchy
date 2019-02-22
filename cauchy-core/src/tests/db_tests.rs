mod db_tests {
    use bytes::Bytes;
    use db::rocksdb::RocksDb;
    use db::storing::*;
    use db::Database;
    use primitives::transaction::*;
    use rocksdb::{Options, DB};
    use std::sync::Arc;

    #[test]
    fn test_rocksdb() {
        let mut opts = Options::default();
        DB::destroy(&opts, ".geodesic/tests/db_a/");

        let key = Bytes::from(&b"testkey"[..]);
        let val = Bytes::from(&b"testval"[..]);
        let db = RocksDb::open_db(".geodesic/tests/db_a/").unwrap();
        db.put(&key, &val).unwrap();
        assert_eq!(db.get(&key).unwrap(), Some(val));
    }

    #[test]
    fn test_put_get_tx() {
        let mut opts = Options::default();
        DB::destroy(&opts, ".geodesic/tests/db_b/");

        let aux = Bytes::from(&b"aux"[..]);
        let binary = Bytes::from(&b"binary"[..]);
        let tx = Transaction::new(1, aux, binary);
        let db = Arc::new(RocksDb::open_db(".geodesic/tests/db_b/").unwrap());
        let tx_id = tx.get_id();
        tx.to_db(db.clone()).unwrap();
        let tx_retrieved = Transaction::from_db(db, &tx_id).unwrap().unwrap();
        assert_eq!(tx, tx_retrieved);
    }

    #[test]
    fn test_put_get_tx_empty() {
        let mut opts = Options::default();
        DB::destroy(&opts, ".geodesic/tests/db_c/");

        let aux = Bytes::from(&b"aux"[..]);
        let binary = Bytes::from(&b"binary"[..]);
        let tx = Transaction::new(1, aux, binary);
        let db = Arc::new(RocksDb::open_db(".geodesic/tests/db_c/").unwrap());
        let tx_id = tx.get_id();
        let tx_retrieved = Transaction::from_db(db, &tx_id);
        assert!(tx_retrieved.unwrap().is_none());
    }
}
