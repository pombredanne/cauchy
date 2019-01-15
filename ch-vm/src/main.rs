extern crate ckb_vm;
extern crate hex;

use std::fs::File;
use std::io::Read;
use std::vec::Vec;

pub mod vm;
use self::vm::VM;

fn main() {
    let mut buffer = Vec::new();
    File::open("tests/sha256_basic")
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

    let input_bytes = b"hello".to_vec();
    let result = vm.run_args(&buffer, input_bytes);
    assert!(result.is_ok());
    println!("Retbytes: {:?}", hex::encode(vm.get_retbytes()));

    let input_bytes = vm.get_retbytes().to_vec();
    let result = vm.run_args(&buffer, input_bytes);
    assert!(result.is_ok());
    println!("Retbytes: {:?}", hex::encode(vm.get_retbytes()));
}
