extern crate hex_literal;
use crypto::hashes::blake2b::Blk2bHashable;
use primitives::{transaction::Transaction, script::*};
use bytes::Bytes;

    #[test]
    fn test_blk2b_basic(){
        let raw = &b"hello world"[..];
        let digest = 	&b"\x02\x1c\xed\x87\x99\x29\x6c\xec\xa5\x57\x83\x2a\xb9\x41\xa5\x0b\x4a\x11\xf8\x34\x78\xcf\x14\x1f\x51\xf9\x33\xf6\x53\xab\x9f\xbc\xc0\x5a\x03\x7c\xdd\xbe\xd0\x6e\x30\x9b\xf3\x34\x94\x2c\x4e\x58\xcd\xf1\xa4\x6e\x23\x79\x11\xcc\xd7\xfc\xf9\x78\x7c\xbc\x7f\xd0"[..];
        assert_eq!(raw.blake2b(), Bytes::from(digest))
    }

    #[test]
    fn test_blk2b_transaction(){
        let raw = &b"\x01\x06\x05hello\x06script\x07world!!"[..];
        let script_a = Script::new(PassBy::Reference, Bytes::from(&b"hello"[..]));
        let script_b = Script::new(PassBy::Value, Bytes::from(&b"script"[..]));
        let script_c = Script::new(PassBy::Value, Bytes::from(&b"world!!"[..]));
        let tx = Transaction::new(1, vec![script_a, script_b, script_c]);
        assert_eq!(tx.blake2b(), raw.blake2b())
    }