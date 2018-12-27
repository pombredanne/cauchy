mod binary_operations {
    use bytes::Bytes;
    use utils::byte_ops::*;

    #[test]
    fn test_and() {
        let raw_a = Bytes::from(&b"\x00\x01\x00\x01"[..]);
        let raw_b = Bytes::from(&b"\x00\x00\x01\x01"[..]);
        let result = Bytes::from(&b"\x00\x00\x00\x01"[..]);
        assert_eq!(Bytes::byte_and(raw_a, raw_b), result)
    }

    #[test]
    fn test_or() {
        let raw_a = Bytes::from(&b"\x00\x01\x00\x01"[..]);
        let raw_b = Bytes::from(&b"\x00\x00\x01\x01"[..]);
        let result = Bytes::from(&b"\x00\x01\x01\x01"[..]);
        assert_eq!(Bytes::byte_or(raw_a, raw_b), result)
    }

    #[test]
    fn test_xor() {
        let raw_a = Bytes::from(&b"\x00\x01\x00\x01"[..]);
        let raw_b = Bytes::from(&b"\x00\x00\x01\x01"[..]);
        let result = Bytes::from(&b"\x00\x01\x01\x00"[..]);
        assert_eq!(Bytes::byte_xor(raw_a, raw_b), result)
    }
}

mod metrics {
    use bytes::Bytes;
    use utils::byte_ops::*;

    #[test]
    fn test_hamming_weight() {
        let raw = Bytes::from(&b"\x00\x01\x00\x01"[..]);
        assert_eq!(raw.hamming_weight(), 2);
    }

    #[test]
    fn test_hamming_distance() {
        let raw_a = Bytes::from(&b"\x00\x01\x00\x01"[..]);
        let raw_b = Bytes::from(&b"\x00\x01\x01\x01"[..]);
        assert_eq!(Bytes::hamming_distance(&raw_a, &raw_b), 1);
    }
}

mod folding {
    use bytes::Bytes;
    use utils::byte_ops::*;

    #[test]
    fn test_fold_a() {
        let raw_a = Bytes::from(&b"\x00\x01\x01\x01\x01\x01"[..]);
        let raw_b = Bytes::from(&b"\x00\x01"[..]);
        assert_eq!(raw_b, raw_a.fold(2).unwrap());
    }

    #[test]
    fn test_fold_b() {
        let raw_a = Bytes::from(&b"\x00\x01\x00\x01"[..]);
        let raw_b = Bytes::from(&b"\x00\x00"[..]);
        assert_eq!(raw_b, raw_a.fold(2).unwrap());
    }

    #[test]
    fn test_fold_c() {
        let raw_a = Bytes::from(&b"\x00\x01\x01\x01\x01\x01"[..]);
        let raw_b = Bytes::from(&b"\x01\x00\x00"[..]);
        assert_eq!(raw_b, raw_a.fold(3).unwrap());
    }

    #[test]
    fn test_fold_d() {
        let raw_a = Bytes::from(&b"\x00\x01\x01\x01\x01\x01"[..]);
        assert_eq!(Err("Not a divisor".to_string()), raw_a.fold(4));
    }
}
