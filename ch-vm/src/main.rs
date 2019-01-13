extern crate ckb_vm;
extern crate hex;

use std::fs::File;
use std::io::Read;
use std::vec::Vec;

pub mod vm;
use self::vm::VM;

use ckb_vm::CoreMachine;

// use std::process::Command;

// fn do_build(sfile : &String)
// {
//     //println!("{:?}", String::from_utf8_lossy(&Command::new("ls").output().unwrap().stdout));
//     let output = Command::new("/opt/riscv/bin/riscv64-unknown-elf-gcc")
//     .arg(sfile)
//     //.arg(format!("-o {}.elf", sfile))
//     .arg("-o test.elf")
//     .output();

//     // println!("{:?}", String::from_utf8_lossy(&output.unwrap().stdout));
//     println!("{:?}", &output.unwrap());
// }

fn main() {
    let mut file = File::open("tests/sha256").unwrap();

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();
    vm.init();

    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    assert!(result.is_ok());

    println!("Retbytes: {:?}", hex::encode(vm.machine.get_retbytes()));

    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec(), b"Hello".to_vec(), b"5".to_vec()]);
    assert!(result.is_ok());

    println!("Retbytes: {:?}", hex::encode(vm.machine.get_retbytes()));
}

#[cfg(test)]


#[test]
fn test_simple() {
    let mut file = File::open("tests/simple").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();
    vm.init();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_vm_syscalls() {
    let mut file = File::open("tests/syscalls").unwrap();

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();
    vm.init();

    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    let bytes = vm.machine.get_retbytes();
    assert_eq!(bytes, &vec![0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48]);
}

#[test]
fn test_sha256() {
    let mut file = File::open("tests/sha256").unwrap();

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();
    vm.init();

    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    assert!(result.is_ok());
    // assert_eq!(result.unwrap(), 1);

    let bytes = vm.machine.get_retbytes();
    assert_eq!(bytes, &hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap());

    // Now test a string as input
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec(), b"Hello".to_vec(), b"5".to_vec()]);
    assert!(result.is_ok());
    // assert_eq!(result.unwrap(), 1);

    let bytes = vm.machine.get_retbytes();
    println!("{:X?}", bytes);
    assert_eq!(bytes, &hex::decode("185F8DB32271FE25F561A6FC938B2E264306EC304EDA518007D1764826381969").unwrap());
    
}
