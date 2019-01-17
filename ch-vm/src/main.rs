extern crate ckb_vm;
extern crate hex;

use std::fs::File;
use std::io::Read;
use std::vec::Vec;
use std::time::{Duration, Instant};

pub mod vm;
use self::vm::VM;

fn main() {
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

    // let mut vm = VM::new();
    // let mut pubkey = hex::decode("e91c69230bd93ccd2c64913e71c0f34ddabbefb4acb3a475eae387621fec89325822d4b15e2b72fd1ffd5b58ff1d726c55b74ce114317c3879547199891d3679").unwrap();
    // let mut sig = hex::decode("166f23ef9c6a5528070dd26ad3b39aeb5f7a7724e7c7c9735c74c0e4a9b820670c6135e5cb51517a461a63cb566a67ec22cb56fda4e4706826e767b1cf37963c").unwrap();
    // let mut msg = hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    // let mut args = vec![];
    // args.append(&mut pubkey);
    // args.append(&mut sig);
    // args.append(&mut msg);
    // let now = Instant::now();
    // let result = vm.run_args(&buffer, args.to_vec() );
    // assert!(result.is_ok());
    // assert_eq!(result.unwrap(), 1);
    // let bytes = vm.get_retbytes();
    // println!("({}s) ecsda_test returns {:?}", now.elapsed().as_secs(), &hex::encode(bytes));

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
