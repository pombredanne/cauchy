mod odd_sketch {
    use bytes::Bytes;
    use crypto::sketches::odd_sketch::*;
    use utils::byte_ops::*;

    #[test]
    fn test_sketchable_permutation() {
        let ele_a = Bytes::from(&b"hello"[..]);
        let ele_b = Bytes::from(&b"script"[..]);
        let ele_c = Bytes::from(&b"world!!"[..]);
        let vec_a = vec![ele_a.clone(), ele_b.clone(), ele_c.clone()];
        let vec_b = vec![ele_b, ele_a, ele_c];
        assert_eq!(
            Bytes::from(vec_a.odd_sketch()),
            Bytes::from(vec_b.odd_sketch())
        )
    }

    #[test]
    fn test_sketchable_size() {
        let ele_a = Bytes::from(&b"hello"[..]);
        let ele_b = Bytes::from(&b"script"[..]);
        let ele_c = Bytes::from(&b"world!!"[..]);
        let ele_d = Bytes::from(&b"extra"[..]);
        let ele_e = Bytes::from(&b"extra2"[..]);
        let vec_a = vec![ele_a, ele_b, ele_c, ele_d, ele_e];
        let sketch_a = vec_a.odd_sketch();
        assert_eq!(sketched_size(&sketch_a), 5)
    }

    #[test]
    fn test_sketchable_symmetric_difference() {
        let script_a = Bytes::from(&b"hello"[..]);
        let script_b = Bytes::from(&b"script"[..]);
        let script_c = Bytes::from(&b"world!!"[..]);
        let script_d = Bytes::from(&b"extra"[..]);
        let script_e = Bytes::from(&b"extra2"[..]);
        let vec_a = vec![script_a.clone(), script_b.clone(), script_c.clone()];
        let vec_b = vec![script_b, script_a, script_d, script_e];
        let sketch_a = vec_a.odd_sketch();
        let sketch_b = vec_b.odd_sketch();
        assert_eq!(sketched_size(&sketch_a.byte_xor(sketch_b)), 3)
    }
}
