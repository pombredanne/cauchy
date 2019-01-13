extern crate ckb_vm;

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
    
    let mut file = File::open("tests/syscalls").unwrap();
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();
    vm.init();

    let result = vm.run(&buffer);
    assert!(result.is_ok());

    println!("Retbytes: {:?}", vm.machine.get_retbytes());
}

#[cfg(test)]

#[test]
fn test_simple() {
    let mut file = File::open("tests/simple").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    
    let mut vm = VM::new();
    vm.init();
    let result = vm.run(&buffer);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_vm_simple() {
    let mut file = File::open("tests/simple").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();
    let result = vm.run(&buffer);

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

    let result = vm.run(&buffer);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

     let bytes = vm.machine.get_retbytes();
    assert_eq!(bytes, &vec![0x41,0x42,0x43,0x44,0x45,0x46,0x47,0x48]);
}