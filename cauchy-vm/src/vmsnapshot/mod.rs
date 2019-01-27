use std::mem::size_of;

use ckb_vm::{
    CoreMachine, Error, SparseMemory, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7,
    RISCV_GENERAL_REGISTER_NUMBER,
};

use ckb_vm::instructions::Register;
use ckb_vm::memory::Page;

use std::fs::File;
use std::io::{Read, Write};

pub struct VMSnapshot<R: Register> {
    pages: Vec<Page>,
    registers: Vec<R>,
    pc: R,
}

impl<R: Register> VMSnapshot<R> {
    pub fn new(machine: &mut CoreMachine<R, SparseMemory>) -> VMSnapshot<R> {
        VMSnapshot {
            pages: machine.memory_mut().pages.to_vec(),
            registers: machine.registers_mut().to_vec(),
            pc: machine.pc(),
        }
    }

    pub fn serialize_reg(&self, reg: &R, bytes: &mut Vec<u8>) {
        for i in 0..size_of::<R>() {
            bytes.push((reg.to_u64() >> (i * 8)) as u8);
        }
    }

    pub fn deserialize_reg(bytes: &[u8], idx: &mut usize) -> u64 {
        let mut res: u64 = 0;
        for i in 0..size_of::<u64>() {
            res += (bytes[i] as u64) << (i * 8);
            *idx += 1;
        }
        res
    }

    pub fn save_to_file(&mut self, fname : String) {
        // let mut buffer = File::create("sysdump.memoryz").unwrap();
        let mut buffer = match File::create(fname) {
            Err(e) => panic!("Could not create file in VMSnapshot::save_to_file()"),
            Ok(f) => f,
        };
        let mut data: Vec<u8> = vec![];

        // Save PC
        self.serialize_reg(&self.pc, &mut data);

        // Serialize registers
        for (i, r) in self.registers.iter().enumerate() {
            self.serialize_reg(r, &mut data);
        }
        buffer.write(&data).unwrap();

        // Dump memory pages
        for p in self.pages.iter() {
            buffer.write(p).unwrap();
        }
    }
}
