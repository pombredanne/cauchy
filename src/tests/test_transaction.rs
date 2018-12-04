mod transaction_tests{
    extern crate bytes;

    use primitives::script::*;
    use bytes::Bytes;
    use primitives::transaction::*;
    use utils::serialisation::*;

    #[test]
    fn test_serialise(){
        let raw = &b"\x06\x05hello\x06script\x07world!!"[..];
        let script_a = Script::new(PassBy::Reference, Bytes::from(&b"hello"[..]));
        let script_b = Script::new(PassBy::Value, Bytes::from(&b"script"[..]));
        let script_c = Script::new(PassBy::Value, Bytes::from(&b"world!!"[..]));
        let tx = Transaction::new(vec![script_a, script_b, script_c]);
        assert_eq!(Bytes::from(tx), Bytes::from(raw))
    }

    #[test]
    fn test_deserialise(){
        let raw = Bytes::from(&b"\x06\x05hello\x06script\x07world!!"[..]);
        let script_a = Script::new(PassBy::Reference, Bytes::from(&b"hello"[..]));
        let script_b = Script::new(PassBy::Value, Bytes::from(&b"script"[..]));
        let script_c = Script::new(PassBy::Value, Bytes::from(&b"world!!"[..]));
        let tx = Transaction::new(vec![script_a, script_b, script_c]);
        let tx_b = Transaction::try_from(raw).unwrap();
        assert_eq!(tx, tx_b)
    }

    #[test]
    fn test_serialise_deserialise(){
        let raw = Bytes::from(&b"\x06\x05hello\x06script\x07world!!"[..]);
        let tx_b = Transaction::try_from(raw.clone()).unwrap();
        assert_eq!(raw, Bytes::from(tx_b))
    }
}