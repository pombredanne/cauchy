extern crate ckb_vm;

use ckb_vm::{
    CoreMachine, Error, SparseMemory, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7,
};

use ckb_vm::memory::{ Memory};
use std::vec::Vec;

pub struct VM {
    ret_bytes : Vec<u8>
}

impl VM {
    pub fn new() -> VM {
        VM {
            ret_bytes : vec![],
        }
    }

    pub fn run(&mut self, buffer: &[u8], args : &[Vec<u8>]) -> Result<u8, Error> {
        // self.machine.run(buffer, args)
        let mut machine = ckb_vm::DefaultMachine::<u64, SparseMemory>::default();
        machine.add_syscall_module(Box::new(VMSyscalls { }));
        let result = machine.run(buffer, args);
        self.ret_bytes = machine.get_retbytes().to_vec();
        result
    }

    pub fn run_args(&mut self, buffer: &[u8], input_bytes : Vec<u8>) -> Result<u8, Error> {
        let len = input_bytes.len();
        let args = &vec![b"__vm_script".to_vec(), input_bytes, len.to_string().as_bytes().to_vec()];
        self.run(buffer, args)
    }

    pub fn get_retbytes(&mut self) -> &Vec<u8> {
        &self.ret_bytes
    }
}


struct VMSyscalls { }

impl Syscalls<u64, SparseMemory> for VMSyscalls {
    fn initialize(&mut self, _machine: &mut CoreMachine<u64, SparseMemory>) -> Result<(), Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut CoreMachine<u64, SparseMemory>) -> Result<bool, Error> {
        match machine.registers()[A7] {
            // For testing purposes, leave it
            1111 => {
                let result = machine.registers()[A0]
                    + machine.registers()[A1]
                    + machine.registers()[A2]
                    + machine.registers()[A3]
                    + machine.registers()[A4]
                    + machine.registers()[A5];
                machine.registers_mut()[A0] = result;
                Ok(true)
            }
            0xCBFF => {
                let sz = machine.registers()[A5];
                let addr = machine.registers()[A6];
                let mut ret_bytes = Vec::<u8>::new();

                for idx in addr..(addr+sz){
                    ret_bytes.push(machine.memory_mut().load8(idx as usize).unwrap());
                }
                machine.store_retbytes(ret_bytes);

                Ok(true)
            }
            0xCBFE => {
                let sz = machine.registers()[A4];
                let addr = machine.registers()[A3];

                // Store out value at address addr
                let store_bytes = hex::decode("DEADBEEF").unwrap();
                machine.memory_mut().store_bytes(addr as usize, &store_bytes).unwrap();

                let mut ret_bytes = Vec::<u8>::new();
                for idx in addr..(addr+sz){
                    ret_bytes.push(machine.memory_mut().load8(idx as usize).unwrap());
                }
                println!("{:X?}", ret_bytes);
                Ok(true)
            }
            _ => Ok(false)
        }
    }
}

#[cfg(test)]

use std::fs::File;
use std::io::Read;

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
    let result = vm.run_args(&buffer, input_bytes);
    assert!(result.is_ok());
    let bytes = vm.get_retbytes();
    assert_eq!(bytes, &hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap());

    // Now test double sha256
    let input_bytes = vm.get_retbytes().to_vec();
    let result = vm.run_args(&buffer, input_bytes);
    assert!(result.is_ok());
    let bytes = vm.get_retbytes();
    assert_eq!(bytes, &hex::decode("9595c9df90075148eb06860365df33584b75bff782a510c6cd4883a419833d50").unwrap());

    let bytes = vm.get_retbytes();
    println!("{:X?}", bytes);
    
}

#[test]
fn test_syscall2() {
    let mut buffer = Vec::new();
    File::open("tests/syscalls2").unwrap().read_to_end(&mut buffer).unwrap();

    let mut vm = VM::new();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    let bytes = vm.get_retbytes();
    assert_eq!(bytes, &hex::decode("DEADBEEF05060708").unwrap());
}