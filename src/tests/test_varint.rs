mod serialisation {
    use primitives::varint::VarInt;
    use bytes::Bytes;
    use utils::serialisation::*;

    #[test]
    fn test_serialise() {
        let v: VarInt = VarInt::new(65535);
        assert_eq!(Bytes::from(&b"\x82\xFE\x7F"[..]), Bytes::from(v));

        let v: VarInt = VarInt::new(16383);
        assert_eq!(Bytes::from(&b"\xFE\x7F"[..]), Bytes::from(v));
    }


    #[test]
    fn test_deserialise() {
        let b = Bytes::from(&b"\xFE\x7F"[..]);
        let v: VarInt = VarInt::try_from(b).unwrap();
        assert_eq!(16383, usize::from(v));
    }

    #[test]
    fn test_serialise_deserialise() {
        fn serialise_deserialise(x: u64) {
            let v1: VarInt = VarInt::new(x);
            let v2: VarInt = VarInt::try_from(Bytes::from(v1.clone())).unwrap();
            assert_eq!(u64::from(v1), u64::from(v2));
        }
        for _ in 0..3000 {
            serialise_deserialise(rand::random::<u64>());
        }
    }

}