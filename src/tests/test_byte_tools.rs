use bytes::Bytes;
use utils::byte_ops::*;

    #[test]
    fn test_and(){
        let raw_a = Bytes::from(&b"\x00\x01\x00\x01"[..]);
        let raw_b = Bytes::from(&b"\x00\x00\x01\x01"[..]);
        let result = Bytes::from(&b"\x00\x00\x00\x01"[..]);
        assert_eq!(Bytes::bitand(raw_a, raw_b), result)
    }

    #[test]
    fn test_or(){
        let raw_a = Bytes::from(&b"\x00\x01\x00\x01"[..]);
        let raw_b = Bytes::from(&b"\x00\x00\x01\x01"[..]);
        let result = Bytes::from(&b"\x00\x01\x01\x01"[..]);
        assert_eq!(Bytes::bitor(raw_a, raw_b), result)
    }

    #[test]
    fn test_xor(){
        let raw_a = Bytes::from(&b"\x00\x01\x00\x01"[..]);
        let raw_b = Bytes::from(&b"\x00\x00\x01\x01"[..]);
        let result = Bytes::from(&b"\x00\x01\x01\x00"[..]);
        assert_eq!(Bytes::bitxor(raw_a, raw_b), result)
    }

    