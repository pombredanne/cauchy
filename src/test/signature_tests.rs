mod operations {
	use crypto::signatures::schnorr::*;
	use bytes::Bytes;
    use crypto::hashes::blake2b::Blk2bHashable;


	#[test]
	fn sign_test_pass() {
		let (sk, pk) = generate_keypair();
		let message = message_from_preimage(&b"hello there"[..]);
		let signature = sign(&message, &sk);
		assert!(verify(&message, &signature, &pk).unwrap());
	}

	#[test]
	fn sign_test_wrong_pk() {
		let (sk, _) = generate_keypair();
		let (_, pk_err) = generate_keypair();
		let message = message_from_preimage(&b"hello there"[..]);
		let signature = sign(&message, &sk);
		assert!(!verify(&message, &signature, &pk_err).unwrap());
	}

	#[test]
	fn sign_test_wrong_message() {
		let (sk, pk) = generate_keypair();
		let message = message_from_preimage(&b"hello there"[..]);
		let message_err = message_from_preimage(&b"hello"[..]);
		let signature = sign(&message, &sk);
		assert!(!verify(&message_err, &signature, &pk).unwrap());
	}

	#[test]
	fn sign_test_wrong_signature() {
		let (sk, pk) = generate_keypair();
		let message = message_from_preimage(&b"hello there"[..]);
		let message_err = message_from_preimage(&b"hello"[..]);
		let signature = sign(&message_err, &sk);
		assert!(!verify(&message, &signature, &pk).unwrap());
	}

}

mod serialisation {
	use crypto::signatures::schnorr::*;
	use secp256k1::Message;


	#[test]
	fn generate_message(){
		let raw = &b"hello world"[..];
		let digest = &b"\x02\x1c\xed\x87\x99\x29\x6c\xec\xa5\x57\x83\x2a\xb9\x41\xa5\x0b\x4a\x11\xf8\x34\x78\xcf\x14\x1f\x51\xf9\x33\xf6\x53\xab\x9f\xbc"[..];
		let message_a = message_from_preimage(raw);
		let message_b = Message::from_slice(digest).expect("32 bytes");
		assert_eq!(message_a, message_b);
	}

	#[test]
	fn pubkey_bytes_convesion(){
		let (_, pk_a) = generate_keypair();
		let pk_raw = pubkey_to_bytes(pk_a.clone());
		let pk_b = bytes_to_pubkey(pk_raw).unwrap();
		assert_eq!(pk_a, pk_b);

	}

}