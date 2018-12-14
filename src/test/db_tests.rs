mod db_tests {
    use bytes::Bytes;
    use db;
    use db::rocksdb::RocksDb;
    use db::Database;
    use utils::constants::TX_DB_PATH;

    #[test]
    fn test_rocksdb() {
        let key = Bytes::from(&b"testkey"[..]);
        let val = Bytes::from(&b"testval"[..]);
        let db = RocksDb::open_db(TX_DB_PATH).unwrap();
        db.put(&key, &val).unwrap();
        assert_eq!(db.get(&key).unwrap(), Some(val));
    }
}
