extern crate ckb_vm;

use ckb_vm::{
    CoreMachine, Error, SparseMemory, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7,
};

use ckb_vm::memory::{ Memory};
use std::vec::Vec;

pub struct VM {
    ret_bytes : Vec<u8>
    // pub machine: ckb_vm::DefaultMachine<'static, u64, SparseMemory>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            // machine: ckb_vm::DefaultMachine::<u64, SparseMemory>::default(),
            ret_bytes : vec![],
        }
    }
    // pub fn init(&mut self) {
    //     self.machine.add_syscall_module(Box::new(VMSyscalls { }));
    // }
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
        println!("{:?}", args);
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
            _ => Ok(false)
        }
    }
}
