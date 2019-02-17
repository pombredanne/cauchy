mod blk2b {
    use bytes::Bytes;
    use crypto::hashes::blake2b::Blk2bHashable;
    use primitives::transaction::Transaction;

    #[test]
    fn test_blk2b_basic() {
        let raw = &b"hello world"[..];
        let digest = &b"\x02\x1c\xed\x87\x99\x29\x6c\xec\xa5\x57\x83\x2a\xb9\x41\xa5\x0b\x4a\x11\xf8\x34\x78\xcf\x14\x1f\x51\xf9\x33\xf6\x53\xab\x9f\xbc"[..];
        assert_eq!(raw.blake2b(), Bytes::from(digest))
    }

    #[test]
    fn test_blk2b_transaction() {
        let raw = Bytes::from(&b"\x01\x03aux\x06binary"[..]);
        let aux = Bytes::from(&b"aux"[..]);
        let binary = Bytes::from(&b"binary"[..]);
        let tx = Transaction::new(1, aux, binary);
        assert_eq!(tx.blake2b(), raw.blake2b())
    }
}
