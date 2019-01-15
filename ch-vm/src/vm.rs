extern crate ckb_vm;

use ckb_vm::{CoreMachine, Error, SparseMemory, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7};

use ckb_vm::memory::Memory;
use std::fs::File;
use std::io::Read;
use std::vec::Vec;

pub struct VM {
    ret_bytes: Vec<u8>,
}

impl VM {
    pub fn new() -> VM {
        VM { ret_bytes: vec![] }
    }

    pub fn run(&mut self, buffer: &[u8], args: &[Vec<u8>]) -> Result<u8, Error> {
        // self.machine.run(buffer, args)
        let mut machine = ckb_vm::DefaultMachine::<u64, SparseMemory>::default();
        machine.add_syscall_module(Box::new(VMSyscalls {}));
        let result = machine.run(buffer, args);
        self.ret_bytes = machine.get_retbytes().to_vec();
        result
    }

    pub fn run_args(&mut self, buffer: &[u8], input_bytes: Vec<u8>) -> Result<u8, Error> {
        let len = input_bytes.len();
        let args = &vec![
            b"__vm_script".to_vec(),
            input_bytes,
            len.to_string().as_bytes().to_vec(),
        ];
        self.run(buffer, args)
    }

    pub fn get_retbytes(&mut self) -> &Vec<u8> {
        &self.ret_bytes
    }
}

struct VMSyscalls {}

impl VMSyscalls {
    fn lookup_script() -> Vec<u8> {
        let mut buffer = Vec::new();
        File::open("tests/sha256")
            .unwrap()
            .read_to_end(&mut buffer)
            .unwrap();
        buffer
    }
}

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
            //  __vm_retbytes(addr, size)
            0xCBFF => {
                let sz = machine.registers()[A5];
                let addr = machine.registers()[A6];
                let mut ret_bytes = Vec::<u8>::new();

                for idx in addr..(addr + sz) {
                    ret_bytes.push(machine.memory_mut().load8(idx as usize).unwrap());
                }
                machine.store_retbytes(ret_bytes);

                Ok(true)
            }
            // __vm_call(sendbuff, sendsize, recvbuff, recvsize)
            0xCBFE => {
                let recv_addr = machine.registers()[A3];
                let recv_sz = machine.registers()[A4];
                let send_addr = machine.registers()[A5];
                let send_sz = machine.registers()[A6];

                // Get the send bytes
                let mut send_bytes = Vec::<u8>::new();
                for idx in send_addr..(send_addr + send_sz) {
                    send_bytes.push(machine.memory_mut().load8(idx as usize).unwrap());
                }
                println!("passing: {:X?} from addr: {:X?}", send_bytes, send_addr);

                // Lookup the script that's being called
                let call_script = &VMSyscalls::lookup_script();

                // Setup a new machine to run the script
                let mut call_machine = ckb_vm::DefaultMachine::<u64, SparseMemory>::default();
                call_machine.add_syscall_module(Box::new(VMSyscalls {}));

                // Get any input bytes intended to be sent to the callable script
                // let input_bytes = call_machine.get_retbytes().to_vec();
                let len = send_bytes.len();
                let args = &vec![
                    b"__vm_script".to_vec(),
                    send_bytes,
                    len.to_string().as_bytes().to_vec(),
                ];
                let result = call_machine.run(call_script, args);
                assert!(result.is_ok());
                let store_bytes = call_machine.get_retbytes().to_vec();

                // Store our value at address addr
                // let store_bytes = hex::decode("DEADBEEF").unwrap();
                machine
                    .memory_mut()
                    .store_bytes(recv_addr as usize, &store_bytes)
                    .unwrap();

                // let mut ret_bytes = Vec::<u8>::new();
                // for idx in recv_addr..(recv_addr + recv_sz) {
                //     ret_bytes.push(machine.memory_mut().load8(idx as usize).unwrap());
                // }
                // println!("{:X?}", ret_bytes);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
// use std::fs::File;
// use std::io::Read;

#[test]
fn test_simple() {
    let mut buffer = Vec::new();
    File::open("tests/simple")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_vm_syscalls() {
    let mut buffer = Vec::new();
    File::open("tests/syscalls")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

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
    File::open("tests/sha256")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    assert!(result.is_ok());
    let bytes = vm.get_retbytes();
    assert_eq!(
        bytes,
        &hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap()
    );

    // Now test a string as input
    let input_bytes = b"hello".to_vec();
    let result = vm.run_args(&buffer, input_bytes);
    assert!(result.is_ok());
    let bytes = vm.get_retbytes();
    assert_eq!(
        bytes,
        &hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap()
    );

    // Now test double sha256
    let input_bytes = vm.get_retbytes().to_vec();
    let result = vm.run_args(&buffer, input_bytes);
    assert!(result.is_ok());
    let bytes = vm.get_retbytes();
    assert_eq!(
        bytes,
        &hex::decode("9595c9df90075148eb06860365df33584b75bff782a510c6cd4883a419833d50").unwrap()
    );

    let bytes = vm.get_retbytes();
    println!("{:X?}", bytes);
}

#[test]
fn test_syscall2() {
    let mut buffer = Vec::new();
    File::open("tests/syscalls2")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    // let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    let result = vm.run_args(&buffer, b"hello".to_vec());
    // assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    let bytes = vm.get_retbytes();
    println!("syscalls2 returns {:X?}", bytes);
    // The return val should be the sha256 hash of "hello"
    assert_eq!(bytes, &hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap());
}

#[test]
fn test_ecdsa() {
    let mut buffer = Vec::new();
    File::open("tests/ecdsa_test")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    // let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    let mut pubkey = hex::decode("57cb298b4766c3992686890c3e6e034fd2cc3d6fd4ec184f2aef8687ae361ef2066271a1bef1bc7b42d506f04b0f63b427bd9d98ea030eea187fc8b431787fa5").unwrap();
    let mut sig = hex::decode("3cbe315ecb178b618a136c6ac9f668daedee16d91c4d9acb4743e1a079ef4a4c441f94eee78fa79f6769dbd1e70862928dff11083dedcf0175870e938e015743").unwrap();
    let mut msg = hex::decode("6d7367").unwrap();
    let mut args = vec![];
    args.append(&mut pubkey);
    args.append(&mut sig);
    args.append(&mut msg);
    let result = vm.run_args(&buffer, args.to_vec() );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    let bytes = vm.get_retbytes();
    println!("ecsda_test returns {:X?}", bytes);
    // assert_eq!(bytes, &vec![133, 11, 22, 45, 153, 51, 103, 207, 200, 145, 35, 70, 37, 74, 148, 41, 96, 193, 130, 4, 182, 109, 218, 180, 239, 222, 188, 120, 59, 118, 236, 216]);
    assert_eq!(bytes, &args);
}