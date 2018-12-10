mod blk2b {
    use crypto::hashes::{blake2b::Blk2bHashable};
    use primitives::{transaction::Transaction, script::Script};
    use bytes::Bytes;

    #[test]
    fn test_blk2b_basic(){
        let raw = &b"hello world"[..];
        let digest = &b"\x02\x1c\xed\x87\x99\x29\x6c\xec\xa5\x57\x83\x2a\xb9\x41\xa5\x0b\x4a\x11\xf8\x34\x78\xcf\x14\x1f\x51\xf9\x33\xf6\x53\xab\x9f\xbc\xc0\x5a\x03\x7c\xdd\xbe\xd0\x6e\x30\x9b\xf3\x34\x94\x2c\x4e\x58\xcd\xf1\xa4\x6e\x23\x79\x11\xcc\xd7\xfc\xf9\x78\x7c\xbc\x7f\xd0"[..];
        assert_eq!(raw.blake2b(), Bytes::from(digest))
    }

    #[test]
    fn test_blk2b_transaction(){
        let raw = &b"\x01\x06\x05hello\x06script\x07world!!"[..];
        let script_a = Script::new(Bytes::from(&b"hello"[..]));
        let script_b = Script::new(Bytes::from(&b"script"[..]));
        let script_c = Script::new(Bytes::from(&b"world!!"[..]));
        let tx = Transaction::new(1, 6, vec![script_a, script_b, script_c]);
        assert_eq!(tx.blake2b(), raw.blake2b())
    }
}

mod odd_sketch {
    use crypto::hashes::oddsketch::*;
    use utils::byte_ops::*;
    use bytes::Bytes;
    use primitives::script::Script;

    #[test]
    fn test_sketchable_permutation(){
        let script_a = Script::new(Bytes::from(&b"hello"[..]));
        let script_b = Script::new(Bytes::from(&b"script"[..]));
        let script_c = Script::new(Bytes::from(&b"world!!"[..]));
        let vec_a = vec![script_a.clone(), script_b.clone(), script_c.clone()];
        let vec_b = vec![script_b, script_a, script_c];
        assert_eq!(Bytes::from(vec_a.odd_sketch()), Bytes::from(vec_b.odd_sketch()))
    }

    #[test]
    fn test_sketchable_size(){
        let script_a = Script::new(Bytes::from(&b"hello"[..]));
        let script_b = Script::new(Bytes::from(&b"script"[..]));
        let script_c = Script::new(Bytes::from(&b"world!!"[..]));
        let script_d = Script::new(Bytes::from(&b"extra"[..]));
        let script_e = Script::new(Bytes::from(&b"extra2"[..]));
        let vec_a = vec![script_a, script_b, script_c, script_d, script_e];
        let sketch_a = vec_a.odd_sketch();
        assert_eq!(sketched_size(sketch_a), 5)
    }

    #[test]
    fn test_sketchable_symmetric_difference(){
        let script_a = Script::new(Bytes::from(&b"hello"[..]));
        let script_b = Script::new(Bytes::from(&b"script"[..]));
        let script_c = Script::new(Bytes::from(&b"world!!"[..]));
        let script_d = Script::new(Bytes::from(&b"extra"[..]));
        let script_e = Script::new(Bytes::from(&b"extra2"[..]));
        let vec_a = vec![script_a.clone(), script_b.clone(), script_c.clone()];
        let vec_b = vec![script_b, script_a, script_d, script_e];
        let sketch_a = vec_a.odd_sketch();
        let sketch_b = vec_b.odd_sketch();
        assert_eq!(sketched_size(sketch_a.byte_xor(sketch_b)), 3)
    }
}