extern crate ckb_vm;
extern crate hex;

use std::fs::File;
use std::io::Read;
use std::vec::Vec;

pub mod vm;
use self::vm::VM;

fn main() {
    let mut buffer = Vec::new();
    File::open("tests/sha256").unwrap().read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();

    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    // let result = vm.run_args(&buffer, b"abc".to_vec());
    assert!(result.is_ok());
    println!("Retbytes: {:?}", hex::encode(vm.get_retbytes()));

    let input_bytes = vm.get_retbytes().to_vec();
    let len = input_bytes.len();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec(), input_bytes, len.to_string().as_bytes().to_vec()]);
    assert!(result.is_ok());
    println!("Retbytes: {:?}", hex::encode(vm.get_retbytes()));


    let input_bytes = b"hello".to_vec();
    let len = input_bytes.len();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec(), input_bytes, len.to_string().as_bytes().to_vec()]);
    assert!(result.is_ok());
    println!("Retbytes: {:?}", hex::encode(vm.get_retbytes()));

    let input_bytes = vm.get_retbytes().to_vec();
    let len = input_bytes.len();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec(), input_bytes, len.to_string().as_bytes().to_vec()]);
    assert!(result.is_ok());
    println!("Retbytes: {:?}", hex::encode(vm.get_retbytes()));


}

#[cfg(test)]


#[test]
fn test_simple() {
    let mut buffer = Vec::new();
    File::open("tests/simple").unwrap().read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_vm_syscalls() {
    let mut buffer = Vec::new();
    File::open("tests/syscalls").unwrap().read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    let bytes = vm.get_retbytes();
    assert_eq!(bytes, &vec![0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48]);
}

#[test]
fn test_sha256() {
    let mut buffer = Vec::new();
    File::open("tests/sha256").unwrap().read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    assert!(result.is_ok());
    let bytes = vm.get_retbytes();
    assert_eq!(bytes, &hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap());

    // Now test a string as input
    let input_bytes = b"hello".to_vec();
    let len = input_bytes.len();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec(), input_bytes, len.to_string().as_bytes().to_vec()]);
    assert!(result.is_ok());
    let bytes = vm.get_retbytes();
    assert_eq!(bytes, &hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap());

    // Now test double sha256
    let input_bytes = vm.get_retbytes().to_vec();
    let len = input_bytes.len();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec(), input_bytes, len.to_string().as_bytes().to_vec()]);
    assert!(result.is_ok());
    let bytes = vm.get_retbytes();
    assert_eq!(bytes, &hex::decode("9595c9df90075148eb06860365df33584b75bff782a510c6cd4883a419833d50").unwrap());

    let bytes = vm.get_retbytes();
    println!("{:X?}", bytes);
    
}
