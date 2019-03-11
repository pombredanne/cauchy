mod oddsketch {
    use bytes::Bytes;
    use crypto::hashes::blake2b::*;
    use crypto::hashes::*;
    use crypto::sketches::odd_sketch::*;
    use crypto::sketches::*;
    use rand::Rng;

    #[derive(Clone)]
    pub struct DummyHolder {
        b: Bytes,
    }

    impl DummyHolder {
        fn new() -> DummyHolder {
            let mut rng = rand::thread_rng();
            let rand_dat: [u8; 8] = rng.gen();
            DummyHolder {
                b: Bytes::from(&rand_dat[..]),
            }
        }
    }

    impl Identifiable for DummyHolder {
        fn get_id(&self) -> Bytes {
            self.b.blake2b()
        }
    }

    #[test]
    fn test_sketchable_permutation() {
        let ele_a = DummyHolder::new();
        let ele_b = DummyHolder::new();
        let ele_c = DummyHolder::new();
        let vec_a = vec![ele_a.clone(), ele_b.clone(), ele_c.clone()];
        let vec_b = vec![ele_b, ele_a, ele_c];
        assert_eq!(OddSketch::sketch(&vec_a), OddSketch::sketch(&vec_b))
    }

    #[test]
    fn test_sketchable_size() {
        let ele_a = DummyHolder::new();
        let ele_b = DummyHolder::new();
        let ele_c = DummyHolder::new();
        let ele_d = DummyHolder::new();
        let ele_e = DummyHolder::new();
        let vec_a = vec![ele_a, ele_b, ele_c, ele_d, ele_e];
        let sketch_a = OddSketch::sketch(&vec_a);
        assert_eq!(sketch_a.size(), 5)
    }

    #[test]
    fn test_sketchable_symmetric_difference() {
        let script_a = DummyHolder::new();
        let script_b = DummyHolder::new();
        let script_c = DummyHolder::new();
        let script_d = DummyHolder::new();
        let script_e = DummyHolder::new();
        let vec_a = vec![script_a.clone(), script_b.clone(), script_c.clone()];
        let vec_b = vec![script_b, script_a, script_d, script_e];
        let sketch_a = OddSketch::sketch(&vec_a);
        let sketch_b = OddSketch::sketch(&vec_b);

        assert_eq!(sketch_a.xor(&sketch_b).size(), 3)
    }
}

mod sketch_interaction {
    use bytes::Bytes;
    use crypto::hashes::blake2b::*;
    use crypto::hashes::*;
    use crypto::sketches::dummy_sketch::*;
    use crypto::sketches::odd_sketch::*;
    use crypto::sketches::*;
    use rand::Rng;

    #[derive(Clone)]
    pub struct DummyHolder {
        b: Bytes,
    }

    impl DummyHolder {
        fn new() -> DummyHolder {
            let mut rng = rand::thread_rng();
            let rand_dat: [u8; 8] = rng.gen();
            DummyHolder {
                b: Bytes::from(&rand_dat[..]),
            }
        }
    }

    impl Identifiable for DummyHolder {
        fn get_id(&self) -> Bytes {
            self.b.blake2b()
        }
    }

    #[test]
    fn test_decode_equivalence() {
        let script_a = DummyHolder::new();
        let script_b = DummyHolder::new();
        let script_c = DummyHolder::new();
        let vec_a = vec![script_a, script_b, script_c];
        let sketch_a = OddSketch::sketch(&vec_a);
        let sketch_b = DummySketch::sketch(&vec_a);

        let decoded_a = &sketch_b.decode().unwrap().0;
        assert_eq!(OddSketch::sketch_ids(decoded_a), sketch_a)
    }

    #[test]
    fn test_xor_decode_equivalence() {
        let script_a = DummyHolder::new();
        let script_b = DummyHolder::new();
        let script_c = DummyHolder::new();
        let script_d = DummyHolder::new();
        let script_e = DummyHolder::new();
        let vec_a = vec![script_a.clone(), script_b, script_c.clone(), script_e];
        let vec_b = vec![script_a, script_c, script_d];
        let oddsketch_a = OddSketch::sketch(&vec_a);
        let oddsketch_b = OddSketch::sketch(&vec_b);
        let dummysketch_a = DummySketch::sketch(&vec_a);
        let dummysketch_b = DummySketch::sketch(&vec_b);

        let (excess, missing) = (dummysketch_a - dummysketch_b).decode().unwrap();
        assert_eq!(
            OddSketch::sketch_ids(&excess).xor(&OddSketch::sketch_ids(&missing)),
            oddsketch_a.xor(&oddsketch_b)
        )
    }
}
