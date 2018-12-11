mod db_tests {
    use bytes::Bytes;
    use db;
    use db::rocksdb::Rocksdb;
    use db::Database;

    #[test]
    fn test_rocksdb() {
        let key = Bytes::from(&b"testkey"[..]);
        let val = Bytes::from(&b"testval"[..]);
        let db = Rocksdb::open_db(db::STATE_DB_PATH).unwrap();
        db.put(&key, &val).unwrap();
        assert_eq!(db.get(&key).unwrap(), Some(val));
    }
}
