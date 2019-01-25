extern crate ckb_vm;
extern crate rand;

use ckb_vm::{
    CoreMachine, Error, SparseMemory, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7,
    RISCV_GENERAL_REGISTER_NUMBER,
};

use ckb_vm::instructions::Register;
use ckb_vm::memory::Page;

use crate::vm::rand::RngCore;
use ckb_vm::memory::Memory;
use rand::rngs::OsRng;
use std::fs::File;
use std::io::{Read, Write};
use std::mem::size_of;
use std::str;
use std::time::SystemTime;
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
        self.run_func(buffer, 0, input_bytes)
    }

    pub fn run_func(
        &mut self,
        buffer: &[u8],
        func_index: u8,
        input_bytes: Vec<u8>,
    ) -> Result<u8, Error> {
        let len = input_bytes.len();
        let args = &vec![
            vec![func_index],
            input_bytes,
            vec![
                len as u8,
                (len >> 8) as u8,
                (len >> 16) as u8,
                (len >> 24) as u8,
            ],
            // vec![len as u8, 0, 0, 0], // len.to_string().as_bytes().to_vec(),
        ];
        // println!("args: {:X?}", &args);
        self.run(buffer, args)
    }

    pub fn get_retbytes(&mut self) -> &Vec<u8> {
        &self.ret_bytes
    }
}

struct VMSnapshot<R: Register> {
    pages: Vec<Page>,
    registers: Vec<R>,
    pc : R,
}

impl<R: Register> VMSnapshot<R> {
    pub fn new(machine: &mut CoreMachine<R, SparseMemory>) -> VMSnapshot<R> {
        VMSnapshot {
            pages: machine.memory_mut().pages.to_vec(),
            registers: machine.registers_mut().to_vec(),
            pc: machine.pc(),
        }
    }

    fn serialize_reg(&self, reg: &R, bytes: &mut Vec<u8>) {
        for i in 0..size_of::<R>() {
            bytes.push( (reg.to_u64() >> (i * 8)) as u8);
        }
    }

    pub fn save_to_file(&mut self) {
        let mut buffer = File::create("sysdump.memoryz").unwrap();

        let mut data: Vec<u8> = vec![];

        // Save PC
        self.serialize_reg(&self.pc, &mut data);

        // Serialize registers
        for r in self.registers.iter() {
            self.serialize_reg(r, &mut data);
        }
        buffer.write(&data).unwrap();

        // Dump memory pages
        for p in self.pages.iter() {
            buffer.write(p).unwrap();
        }
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
                let txid_addr = machine.registers()[A2];
                let recv_addr = machine.registers()[A3];
                let recv_sz = machine.registers()[A4];
                let send_addr = machine.registers()[A5];
                let send_sz = machine.registers()[A6];
                let func_index = 0;

                // Get TXID we're trying to call
                let mut txid = Vec::<u8>::new();
                for idx in txid_addr..(txid_addr + 64) {
                    txid.push(machine.memory_mut().load8(idx as usize).unwrap());
                }
                // println!("txid: {:X?} from addr: {:X?}", &hex::decode(txid).unwrap(), txid_addr);
                println!(
                    "txid: {:X?} from addr: {:X?}",
                    hex::encode(hex::decode(txid).unwrap()),
                    txid_addr
                );

                // Get the send bytes
                let mut send_bytes = Vec::<u8>::new();
                for idx in send_addr..(send_addr + send_sz) {
                    send_bytes.push(machine.memory_mut().load8(idx as usize).unwrap());
                }
                println!(
                    "passing: {:X?} from addr: {:X?}",
                    str::from_utf8(&send_bytes).unwrap(),
                    send_addr
                );

                // Lookup the script that's being called
                let call_script = &VMSyscalls::lookup_script();

                // Setup a new machine to run the script
                let mut call_machine = ckb_vm::DefaultMachine::<u64, SparseMemory>::default();
                call_machine.add_syscall_module(Box::new(VMSyscalls {}));

                // Get any input bytes intended to be sent to the callable script
                let len = send_bytes.len();
                let args = &vec![
                    vec![func_index],
                    send_bytes,
                    vec![
                        len as u8,
                        (len >> 8) as u8,
                        (len >> 16) as u8,
                        (len >> 24) as u8,
                    ],
                ];
                let result = call_machine.run(call_script, args);
                assert!(result.is_ok());
                let store_bytes = call_machine.get_retbytes().to_vec();

                // Store our value at address addr
                machine
                    .memory_mut()
                    .store_bytes(recv_addr as usize, &store_bytes)
                    .unwrap();

                Ok(true)
            }
            0xCBFD => {
                let mut rng = match OsRng::new() {
                    Ok(g) => g,
                    Err(e) => panic!("Failed to obtain OS RNG: {}", e),
                };
                let sz = machine.registers()[A5];
                let addr = machine.registers()[A6];
                let mut rng_bytes = vec![0; sz as usize];
                rng.fill_bytes(&mut rng_bytes);
                // println!("{:X?}", rng_bytes);
                machine
                    .memory_mut()
                    .store_bytes(addr as usize, &rng_bytes)
                    .unwrap();
                Ok(true)
            }
            // __vm_gettime()
            0xCBFC => {
                let addr = machine.registers()[A5];
                let t = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                machine
                    .memory_mut()
                    .store_bytes(
                        addr as usize,
                        &vec![t as u8, (t >> 8) as u8, (t >> 16) as u8, (t >> 24) as u8],
                    )
                    .unwrap();
                Ok(true)
            }
            // __vm_receive_wait()
            0xCBFB => {
                let addr = machine.registers()[A5];
                //machine.memory_mut().dump_to_file("memory.sysdump".to_string());
                let mut snapshot = VMSnapshot::new(machine);
                snapshot.save_to_file();
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

    // Now test hashing the binary itself
    let mut buffer = Vec::new();
    File::open("tests/sha256")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();
    let result = vm.run_args(&buffer, buffer.to_vec());
    assert!(result.is_ok());
    let bytes = vm.get_retbytes();
    assert_eq!(
        bytes,
        &hex::decode("27a5e5b657f6ba928e8e50059b1c04d45bb4c1d1f963531da44c3f59fec94555").unwrap()
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
    assert_eq!(
        bytes,
        // &hex::decode("87c5158b276f5afd0709709edb49a8b3a9e38de5b9e54ca9dcce8e94b7f631d1").unwrap()
        &hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap()
    );
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
    let mut pubkey = hex::decode("927d42216ae79f7599a50e1204da87cf7fce8fe278773ddd9348393b7ee4d714098fd88fba58bd0e014023118858f67e2294719b53deb1546edf7c3440fefe9f").unwrap();
    let mut sig = hex::decode("84a28969215b235bcf00cc11330a20198f71a0b51f71badccd535dfbaf776cd1c25e7e75fccaff0f14546d9b2f33d6d7d351590f6590a0c682ac0d8422025edc").unwrap();
    let mut msg =
        hex::decode("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad").unwrap();
    let mut args = vec![];
    args.append(&mut pubkey);
    args.append(&mut sig);
    args.append(&mut msg);
    let result = vm.run_func(&buffer, 2, args);
    assert!(result.is_ok());
    // assert if sig verify fails
    assert_eq!(result.unwrap(), 2);
}

#[test]
fn test_time() {
    let mut buffer = Vec::new();
    File::open("tests/time")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    let result = vm.run_func(&buffer, 0, vec![]);
    println!("{:X?}", &hex::encode(&vm.get_retbytes()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_freeze() {
    let mut buffer = Vec::new();
    File::open("tests/freeze")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new();
    let result = vm.run_func(&buffer, 0, vec![]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0xFF);
}
