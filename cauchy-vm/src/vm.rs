extern crate ckb_vm;
use bytes::Bytes;
use core::db::rocksdb::RocksDb;
use core::db::storing::*;
use core::db::*;
use std::sync::Arc;

use ckb_vm::{
    CoreMachine, DefaultCoreMachine, DefaultMachineBuilder, Memory, Register, Error, SparseMemory, SupportMachine, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7,
};


pub struct VM {
    script: Bytes,
    message: Bytes,
    timestamp: u64,
    tx_db: Arc<RocksDb>,
}

impl VM {
    pub fn new(script: Bytes, message: Bytes, timestamp: u64, db: Arc<RocksDb> ) -> VM {
        VM {
            script: script,
            message: message,
            timestamp: timestamp,
            tx_db: db,
        }
    }

    pub fn run(self) -> Result<u8, Error> {
        let mut machine =
        DefaultMachineBuilder::<DefaultCoreMachine<u64, SparseMemory<u64>>>::default()
            .syscall(Box::new(CustomSyscall {}))
            .build();
        machine = machine
            .load_program(&self.script[..], &vec![b"syscall".to_vec()])
            .unwrap();
        let result = machine.interpret();
        result
    }
}

pub struct CustomSyscall {
}

impl<Mac: SupportMachine> Syscalls<Mac> for CustomSyscall {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, Error> {
        let code = &machine.registers()[A7];
        let code = code.to_i32();
        match code {
            1111 => {
                panic!("PC calculation in VM::resume() is incorrect");
            }
            //  __vm_retbytes(addr, size)
            0xCBFF => {
                let sz = machine.registers()[A5].to_u32();
                let addr = machine.registers()[A6].to_u32();
                let mut ret_bytes = Vec::<u8>::new();

                for idx in addr..(addr + sz) {
                    ret_bytes.push(machine.memory_mut().load8(&Mac::REG::from_u32(idx)).unwrap().to_u8());
                }
                machine.store_retbytes(ret_bytes);
                Ok(true)
            }
            _ => Ok(false)
        }
    }
}
