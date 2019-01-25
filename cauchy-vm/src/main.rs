extern crate ckb_vm;
extern crate hex;

use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};
use std::vec::Vec;


pub mod vm;
use self::vm::VM;

fn main() {
    
    // let (sk, pk) = gen_keypair_onchain();
    let sk = hex::decode("a901c2899091c75bf2cc7e2540d855ea8faa1b09b4f4f02528a6427d454decbc").unwrap();
    let pk = hex::decode("1f63d51b1420c5d7bdd19aef42ceeb4fc4fe3403f3c9ea7c3b72c8731e96d2d9bdc8536df25ac575eaf25b0fe6522984889840b941a9d6fde79e7cb21b512b6f").unwrap();
    println!(
        "SK:  {:X?}\nPK:  {:X?}",
        &hex::encode(&sk),
        &hex::encode(&pk)
    );

    let mut buffer = Vec::new();
    File::open("scripts/sha256").unwrap().read_to_end(&mut buffer).unwrap();

    // let buffer = b"abc".to_vec();
    // println!("binary size: {:?}", buffer.len());
    let hash = gen_sha256(&buffer);
    println!("hsh: {:X?}", &hex::encode(&hash));

    let sig = gen_sig(&sk, &hash);
    println!("Sig: {:X?}", &hex::encode(&sig));

    let verified = verify_sig(&pk, &sig, &hash);
    if (verified) {
        println!("Sig verified!");
    } else {
        println!("Sig verify failed :-(");
    }
}

fn verify_sig(pubkey: &Vec<u8>, sig: &Vec<u8>, hash: &Vec<u8>) -> bool {
    let mut buffer = Vec::new();
    File::open("scripts/ecdsa")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    let mut input_bytes = Vec::new();
    input_bytes.append(&mut pubkey.to_vec());
    input_bytes.append(&mut sig.to_vec());
    input_bytes.append(&mut hash.to_vec());

    let retval = vm.run_func(&buffer, 2, input_bytes);
    assert!(retval.is_ok());
    retval.unwrap() == 2
}

fn gen_sig(privkey: &Vec<u8>, hash: &Vec<u8>) -> Vec<u8> {
    let mut buffer = Vec::new();
    File::open("scripts/ecdsa")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    let mut input_bytes = Vec::new();
    input_bytes.append(&mut privkey.to_vec());
    input_bytes.append(&mut hash.to_vec());
    let retval = vm.run_func(&buffer, 1, input_bytes);
    assert!(retval.is_ok());
    assert_eq!(retval.unwrap(), 1);
    vm.get_retbytes().to_vec()
}

fn gen_sha256(bytes: &Vec<u8>) -> Vec<u8> {
    let mut buffer = Vec::new();
    File::open("scripts/sha256")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    let retval = vm.run_func(&buffer, 0, bytes.to_vec());
    assert!(retval.is_ok());
    // println!("sha retval: {:?}", retval.unwrap());
    vm.get_retbytes().to_vec()
}

fn gen_keypair_onchain() -> (Vec<u8>, Vec<u8>) {
    let mut buffer = Vec::new();
    File::open("scripts/ecdsa")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    let retval = vm.run_func(&buffer, 0, vec![]);
    assert!(retval.is_ok());
    assert_eq!(retval.unwrap(), 0);
    let ret_bytes = vm.get_retbytes();
    let privkey = ret_bytes[..32].to_vec();
    let pubkey = ret_bytes[32..32 + 64].to_vec();

    assert_eq!(privkey.len(), 32);
    assert_eq!(pubkey.len(), 64);
    (privkey, pubkey)
}

fn misc_tests() {
    let mut buffer = Vec::new();
    File::open("tests/sha256")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();

    let result = vm.run_args(&buffer, vec![]);
    assert!(result.is_ok());
    println!("Retbytes: {:?}", hex::encode(vm.get_retbytes()));

    let result = vm.run_args(&buffer, b"hello".to_vec());
    assert!(result.is_ok());
    println!("Retbytes: {:?}", hex::encode(vm.get_retbytes()));

    let input_bytes = vm.get_retbytes().to_vec();
    let result = vm.run_args(&buffer, input_bytes);
    assert!(result.is_ok());
    println!("Retbytes: {:?}", hex::encode(vm.get_retbytes()));

    let mut buffer = Vec::new();
    File::open("tests/ecdsa_test")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    let mut pubkey = hex::decode("e91c69230bd93ccd2c64913e71c0f34ddabbefb4acb3a475eae387621fec89325822d4b15e2b72fd1ffd5b58ff1d726c55b74ce114317c3879547199891d3679").unwrap();
    let mut sig = hex::decode("166f23ef9c6a5528070dd26ad3b39aeb5f7a7724e7c7c9735c74c0e4a9b820670c6135e5cb51517a461a63cb566a67ec22cb56fda4e4706826e767b1cf37963c").unwrap();
    let mut msg =
        hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    let mut args = vec![];
    args.append(&mut pubkey);
    args.append(&mut sig);
    args.append(&mut msg);
    let now = Instant::now();
    let result = vm.run_args(&buffer, args.to_vec());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
    let bytes = vm.get_retbytes();
    println!(
        "({}s) ecsda_test returns {:?}",
        now.elapsed().as_secs(),
        &hex::encode(bytes)
    );

    let mut buffer = Vec::new();
    File::open("tests/syscalls2")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();

    let result = vm.run_args(&buffer, b"hello".to_vec());
    assert!(result.is_ok());
    println!("Retbytes: {:?}", hex::encode(vm.get_retbytes()));
}
