mod serialisation {
    extern crate bytes;

    use bytes::Bytes;
    use primitives::transaction::*;
    use utils::serialisation::*;

    #[test]
    fn test_serialise() {
        let raw = &b"\x01\x03aux\x06binary"[..];
        let aux = Bytes::from(&b"aux"[..]);
        let binary = Bytes::from(&b"binary"[..]);
        let tx = Transaction::new(1, aux, binary);
        assert_eq!(Bytes::from(tx), Bytes::from(raw))
    }

    #[test]
    fn test_deserialise() {
        let raw = Bytes::from(&b"\x01\x03aux\x06binary"[..]);
        let aux = Bytes::from(&b"aux"[..]);
        let binary = Bytes::from(&b"binary"[..]);
        let tx = Transaction::new(1, aux, binary);
        let tx_b = Transaction::try_from(raw).unwrap();
        assert_eq!(tx, tx_b)
    }

    #[test]
    fn test_serialise_deserialise() {
        let raw = Bytes::from(&b"\x01\x03aux\x06binary"[..]);
        let tx_b = Transaction::try_from(raw.clone()).unwrap();
        assert_eq!(raw, Bytes::from(tx_b))
    }
}
