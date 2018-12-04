mod script_tests{
    use db;
    use db::Database;
    use db::rocksdb::Rocksdb;
    use bytes::Bytes;


    #[test]
    fn test_rocksdb() {
        let key = Bytes::from(&b"testkey"[..]);
        let val = Bytes::from(&b"testval"[..]);
        let db = Rocksdb::open_db(db::STATE_DB_PATH).unwrap();
        db.put(&key, &val).unwrap();
        assert_eq!(db.get(&key).unwrap(), Some(val));
    }
}