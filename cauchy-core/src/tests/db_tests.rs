mod db_tests {
    use std::sync::Arc;

    use bytes::Bytes;

    use crate::{
        crypto::hashes::*,
        db::{mongodb::MongoDB, storing::*, Database, DataType},
        primitives::transaction::*,
    };

    #[test]
    fn test_mongodb() {

        let key = Bytes::from(&b"testkey"[..]);
        let val = Bytes::from(&b"testval"[..]);
        let db = MongoDB::open_db("tests_db_a").unwrap();
        db.put(&DataType::TX, &key, &val).unwrap();
        assert_eq!(db.get(&DataType::TX, &key).unwrap(), Some(val));
    }

    #[test]
    fn test_put_get_tx() {
        let aux = Bytes::from(&b"aux"[..]);
        let binary = Bytes::from(&b"binary"[..]);
        let tx = Transaction::new(1, aux, binary);
        let db = Arc::new(MongoDB::open_db("tests_db_b").unwrap());
        let tx_id = tx.get_id();
        tx.to_db(db.clone()).unwrap();
        let tx_retrieved = Transaction::from_db(db, &tx_id).unwrap().unwrap();
        assert_eq!(tx, tx_retrieved);
    }

    #[test]
    fn test_put_get_tx_empty() {
        let aux = Bytes::from(&b"aux"[..]);
        let binary = Bytes::from(&b"binary"[..]);
        let tx = Transaction::new(1, aux, binary);
        let db = Arc::new(MongoDB::open_db("tests_db_c").unwrap());
        let tx_id = tx.get_id();
        let tx_retrieved = Transaction::from_db(db, &tx_id);
        assert!(tx_retrieved.unwrap().is_none());
    }
}
