mod db_tests {
    #[macro_use(bson, doc)]
    use std::sync::Arc;
    use bson::spec::BinarySubtype;
    use bson::*;
    use bytes::Bytes;

    use crate::{
        crypto::hashes::*,
        db::{mongodb::MongoDB, storing::*, DataType, Database},
        primitives::transaction::*,
    };

    use bson::{bson, doc};

    #[test]
    fn test_mongodb() {
        let db = MongoDB::open_db("tests_db_a").unwrap();
        db.dropall(&DataType::TX);
        db.put(&DataType::TX, doc! {"_id" : 1, "val" : 1234})
            .unwrap();
        assert_eq!(
            db.get(&DataType::TX, doc! {}).unwrap(),
            Some(doc! { "_id" : 1, "val" => 1234})
        );
        assert_eq!(
            db.update(
                &DataType::TX,
                doc! { "_id" : 1},
                doc! { "$set" : {"val" : 4321}}
            )
            .unwrap(),
            1
        );
        assert_eq!(
            db.get(&DataType::TX, doc! {}).unwrap(),
            Some(doc! { "_id" : 1, "val" => 4321})
        );
    }

    #[test]
    fn test_put_get_tx() {
        let aux = Bytes::from(&b"aux"[..]);
        let binary = Bytes::from(&b"binary"[..]);
        let tx = Transaction::new(1, aux, binary);
        let mut db = MongoDB::open_db("tests_db_b").unwrap();
        db.dropall(&DataType::TX);
        let tx_id = tx.get_id();
        tx.to_db(&mut db.clone(), None).unwrap();
        let tx_retrieved = Transaction::from_db(&mut db, tx_id).unwrap().unwrap();
        assert_eq!(tx, tx_retrieved);
    }

    #[test]
    fn test_put_get_tx_empty() {
        let aux = Bytes::from(&b"aux"[..]);
        let binary = Bytes::from(&b"binary"[..]);
        let tx = Transaction::new(1, aux, binary);
        let mut db = MongoDB::open_db("tests_db_c").unwrap();
        db.dropall(&DataType::TX);
        let tx_id = tx.get_id();
        let tx_retrieved = Transaction::from_db(&mut db, tx_id);
        assert!(tx_retrieved.unwrap().is_none());
    }

    #[test]
    fn test_ordering() {
        let db = MongoDB::open_db("tests_db_d").unwrap();
        db.dropall(&DataType::TX);
        for i in 0..10 {
            let doc = doc! { "_id" : Bson::Binary(BinarySubtype::Generic, (i as u64).to_be_bytes().to_vec() ), "t" : Bson::Binary(BinarySubtype::Generic, (255 as u64).to_be_bytes().to_vec() ), "v" : i+100};
            db.put(&DataType::TX, doc).unwrap();
        }
        for i in 0..10 {
            let doc = doc! { "_id" : Bson::Binary(BinarySubtype::Generic, (i+10 as u64).to_be_bytes().to_vec() ), "t" : Bson::Binary(BinarySubtype::Generic, (1 as u64).to_be_bytes().to_vec() ), "v" : i+100};
            db.put(&DataType::TX, doc).unwrap();
        }
        match db.get(&DataType::TX, doc!{ "t" : Bson::Binary(BinarySubtype::Generic, (255 as u64).to_be_bytes().to_vec() )} ) {
            Ok(Some(some)) => {
                // The result must be the most "recent"
                assert_eq!(some, doc!{ "_id": Bson::Binary(BinarySubtype::Generic, (9 as u64).to_be_bytes().to_vec() ), "t" : Bson::Binary(BinarySubtype::Generic, (255 as u64).to_be_bytes().to_vec() ), "v" : 109} );
            }
            _ => assert!(false)
        }
        let update_doc = doc! { "$unset" : {"v" : ""} };
        let res = db.update(&DataType::TX, doc!{ "t" : Bson::Binary(BinarySubtype::Generic, (255 as u64).to_be_bytes().to_vec() ) } , update_doc);
        // We should have updated 10 of the 20 records, removing their "v" field
        assert_eq!(res.unwrap(), 10);
    }
}
