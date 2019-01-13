extern crate ckb_vm;

use ckb_vm::{
    CoreMachine, Error, SparseMemory, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7,
};

use ckb_vm::memory::{Memory};
use std::vec::Vec;

pub struct VM {
    pub val: u8,
    pub machine: ckb_vm::DefaultMachine<'static, u64, SparseMemory>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            val: 0,
            machine: ckb_vm::DefaultMachine::<u64, SparseMemory>::default(),
        }
    }
    pub fn init(&mut self) {
        self.machine.add_syscall_module(Box::new(VMSyscalls { }));
    }
    pub fn run(&mut self, buffer: &[u8], args : &[Vec<u8>]) -> Result<u8, Error> {
        self.machine.run(buffer, args)
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
            _ => Ok(false)
        }
    }
}
