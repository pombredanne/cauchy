extern crate ckb_vm;
extern crate rand;
extern crate sha2;

use ckb_vm::{
    CoreMachine, DefaultMachine, Error, SparseMemory, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7,
    DEFAULT_STACK_SIZE, RISCV_MAX_MEMORY,
};

use ckb_vm::instructions::Register;

use crate::vm::rand::RngCore;
use ckb_vm::memory::Memory;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::str;
use std::time::SystemTime;
use std::vec::Vec;

use bytes::Bytes;
use core::db::rocksdb::RocksDb;
use core::db::storing::*;
use core::db::*;
use core::primitives::transaction::Transaction;
use std::sync::Arc;

use crate::vmsnapshot::VMSnapshot;

pub struct VM {
    ret_bytes: Vec<u8>,
    txid: Vec<u8>,
    tx_db: Arc<RocksDb>,
}

impl VM {
    pub fn new(tx_db: Arc<RocksDb>) -> VM {
        VM {
            ret_bytes: vec![],
            txid: vec![],
            tx_db,
        }
    }

    pub fn txid_set(&mut self, txid: &[u8]) {
        self.txid = txid.to_vec();
    }

    pub fn run(&mut self, buffer: &[u8], args: &[Vec<u8>]) -> Result<u8, Error> {
        let mut hasher = Sha256::new();
        hasher.input(&buffer);
        self.txid = hasher.result().to_vec();
        let mut machine = DefaultMachine::<u64, SparseMemory>::default();
        machine.add_syscall_module(Box::new(VMSyscalls {
            txid: self.txid.to_vec(),
            tx_db: self.tx_db.clone(),
        }));
        let result = machine.run(buffer, args);
        self.ret_bytes = machine.get_retbytes().to_vec();
        // machine
        //     .memory_mut()
        //     .dump_to_file("machine.memoryz".to_string());
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
        ];
        self.run(buffer, args)
    }

    pub fn get_retbytes(&mut self) -> &Vec<u8> {
        &self.ret_bytes
    }

    pub fn resume(&mut self, buffer: &[u8], args: &[u8]) -> Result<u8, Error> {
        let mut hasher = Sha256::new();
        hasher.input(&buffer);
        self.txid = hasher.result().to_vec();

        let mut machine = ckb_vm::DefaultMachine::<u64, SparseMemory>::default();
        machine.load_elf(&buffer).unwrap();
        machine.add_syscall_module(Box::new(VMSyscalls {
            txid: self.txid.to_vec(),
            tx_db: self.tx_db.clone(),
        }));
        machine.initialize_stack(
            &vec![],
            RISCV_MAX_MEMORY - DEFAULT_STACK_SIZE,
            DEFAULT_STACK_SIZE,
        )?;

        let recv_addr = VMSnapshot::<u64>::load_from_file(&self.txid, &mut machine);

        // Store input args at machine's snapshot recv addr
        machine
            .memory_mut()
            .store_bytes(recv_addr as usize, args)
            .unwrap();

        let result = machine.resume();
        self.ret_bytes = machine.get_retbytes().to_vec();
        result
    }
}

pub trait Retbytes {
    fn get_retbytes(&mut self) -> &Vec<u8>;
    fn store_retbytes(&mut self, retbytes: Vec<u8>);
}

struct VMSyscalls {
    txid: Vec<u8>,
    tx_db: Arc<RocksDb>,
}

impl VMSyscalls {
    fn lookup_script(txid: &String) -> Vec<u8> {
        let mut buffer = Vec::new();
        match File::open(format!("scripts/{}", txid)) {
            Err(e) => panic!("Unable to load txid {}", txid),
            Ok(mut r) => r.read_to_end(&mut buffer).unwrap(),
        };
        // let t = Transaction::from_id();
        buffer
    }

    fn lookup_tx(&self, tx_db: Arc<RocksDb>, txid: &Bytes) -> Transaction {
        Transaction::from_db(tx_db, txid).unwrap().unwrap()
    }
}

impl Syscalls<u64, SparseMemory> for VMSyscalls {
    fn initialize(&mut self, _machine: &mut CoreMachine<u64, SparseMemory>) -> Result<(), Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut CoreMachine<u64, SparseMemory>) -> Result<bool, Error> {
        match machine.registers()[A7] {
            // Used to detect invalid PC assignment during resume
            1111 => {
                panic!("PC calculation in VM::resume() is incorrect");
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

                // Get the send bytes
                let mut send_bytes = Vec::<u8>::new();
                for idx in send_addr..(send_addr + send_sz) {
                    send_bytes.push(match machine.memory_mut().load8(idx as usize) {
                        Err(e) => panic!("sendbuff out of range in __vm_call()! {:?}", e),
                        Ok(r) => r,
                    });
                }

                // Lookup the script that's being called, which is  encoded as a string
                println!(
                    "Looking up script {}",
                    &String::from_utf8(txid.to_vec()).unwrap()
                );
                let call_script = &VMSyscalls::lookup_script(&String::from_utf8(txid).unwrap());

                let mut vm = VM::new(self.tx_db.clone());
                let mut retcode: u8 = 0;

                // // Get the state ID for this script, which should be the TXID for now
                let mut hasher = Sha256::new();
                hasher.input(&call_script);
                let txid = hasher.result().to_vec();

                if VMSnapshot::<u64>::has_saved_state(&txid) {
                    // saved state, resume it
                    println!("Loading state {}", &hex::encode(&txid));
                    println!("Sending bytes {:X?}", &send_bytes);
                    let result = vm.resume(call_script, &send_bytes);
                    retcode = result.unwrap();
                    assert!(result.is_ok());
                } else {
                    // No saved state here, spin up a fresh one
                    println!("Saved state not found for {}", &hex::encode(&txid));
                    let result = vm.run_func(call_script, func_index, send_bytes);
                    assert!(result.is_ok());
                    retcode = result.unwrap();
                }
                let store_bytes = vm.get_retbytes().to_vec();

                // Store our value at address addr
                machine
                    .memory_mut()
                    .store_bytes(recv_addr as usize, &store_bytes)
                    .unwrap();

                println!(
                    "Called script returned {:?} with retcode {:}",
                    &hex::encode(&store_bytes),
                    retcode
                );

                Ok(true)
            }
            // __vm_getrand()
            0xCBFD => {
                let mut rng = match OsRng::new() {
                    Ok(g) => g,
                    Err(e) => panic!("Failed to obtain OS RNG: {}", e),
                };
                let sz = machine.registers()[A5];
                let addr = machine.registers()[A6];
                let mut rng_bytes = vec![0; sz as usize];
                rng.fill_bytes(&mut rng_bytes);
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
                let mut snapshot = VMSnapshot::new(machine);
                snapshot.save_to_file(hex::encode(&self.txid));
                println!("Saved state {:?}", &hex::encode(&self.txid));
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
    let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_simple/").unwrap();

    let mut buffer = Vec::new();
    File::open("tests/simple")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new(Arc::new(tx_db));
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_vm_syscalls() {
    let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_syscalls/").unwrap();

    let mut buffer = Vec::new();
    File::open("tests/syscalls")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new(Arc::new(tx_db));
    let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    let bytes = vm.get_retbytes();
    assert_eq!(bytes, &vec![0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48]);
}

#[test]
fn test_sha256() {
    let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_sha256/").unwrap();

    let mut buffer = Vec::new();
    File::open("tests/sha256")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new(Arc::new(tx_db));
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
    let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_syscall2/").unwrap();

    let mut buffer = Vec::new();
    File::open("tests/syscalls2")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new(Arc::new(tx_db));
    // let result = vm.run(&buffer, &vec![b"__vm_script".to_vec()]);
    let result = vm.run_args(&buffer, b"hello".to_vec());
    // assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    let bytes = vm.get_retbytes();
    println!("syscalls2 returns {:X?}", bytes);
    // The return val should be the sha256 hash of "hello"
    assert_eq!(
        bytes,
        &hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap()
    );
}

#[test]
fn test_ecdsa() {
    let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_ecdsa/").unwrap();

    let mut buffer = Vec::new();
    File::open("tests/ecdsa_test")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new(Arc::new(tx_db));
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
    let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_time/").unwrap();

    let mut buffer = Vec::new();
    File::open("tests/time")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new(Arc::new(tx_db));
    let result = vm.run_func(&buffer, 0, vec![]);
    println!("{:X?}", &hex::encode(&vm.get_retbytes()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_freeze() {
    let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_freeze/").unwrap();
    let arc_tx_db = Arc::new(tx_db);

    let mut buffer = Vec::new();
    File::open("tests/freeze")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new(arc_tx_db.clone());
    let result = vm.run_func(&buffer, 0, vec![]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 123);

    let mut buffer = Vec::new();
    match File::open("tests/freeze_call") {
        Err(e) => panic!("Unable to open file in test_freeze() {:?}", e),
        Ok(mut r) => r.read_to_end(&mut buffer).unwrap(),
    };

    let mut vm = VM::new(arc_tx_db);
    let result = vm.run_args(&buffer, b"DEADBEEF".to_vec());
    if !(result.is_ok()) {
        panic!("{:?}", result.err());
    }
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_simple_contract() {
    let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_simple_contract/").unwrap();

    // let mut buffer = Vec::new();
    // File::open("tests/simple_contract")
    //     .unwrap()
    //     .read_to_end(&mut buffer)
    //     .unwrap();

    // let mut vm = VM::new();
    // let result = vm.run_func(&buffer, 0, vec![]);

    // println!("contract: {:X?}", &hex::encode(&vm.get_retbytes()));
    // assert!(result.is_ok());
    // assert_eq!(result.unwrap(), 123);

    let mut buffer = Vec::new();
    File::open("tests/simple_contract_call")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new(Arc::new(tx_db));
    let result = vm.run_func(&buffer, 0, vec![]);
    println!("contract_call: {:X?}", &hex::encode(&vm.get_retbytes()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_simple_contract_cpp() {
    let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_simple_contract_cpp/").unwrap();

    let mut buffer = Vec::new();
    File::open("tests/simple_contract_cpp")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut vm = VM::new(Arc::new(tx_db));
    let result = vm.run_func(&buffer, 0, vec![]);
    println!("contract_call: {:X?}", &hex::encode(&vm.get_retbytes()));
    assert_eq!(result.unwrap(), 1);
    assert!(result.is_ok());
}
