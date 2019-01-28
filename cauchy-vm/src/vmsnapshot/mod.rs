use std::mem::size_of;

use ckb_vm::{
    CoreMachine, Error, SparseMemory, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7,
    RISCV_GENERAL_REGISTER_NUMBER, RISCV_PAGESIZE
};

use ckb_vm::instructions::Register;
use ckb_vm::memory::{ Page, Memory};

use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};

pub struct VMSnapshot<R: Register> {
    pages: Vec<Page>,
    registers: Vec<R>,
    pc: R,
    recv_addr: R,
}

impl<R: Register> VMSnapshot<R> {
    pub fn new(machine: &mut CoreMachine<R, SparseMemory>) -> VMSnapshot<R> {
        VMSnapshot {
            pages: machine.memory_mut().pages.to_vec(),
            registers: machine.registers_mut().to_vec(),
            pc: machine.pc(),
            recv_addr: machine.registers()[A5],
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

    pub fn load_from_file(txid: &[u8], machine: &mut CoreMachine<R, SparseMemory>) -> R {
        
        // Load the state into a buff
        let mut buffer = Vec::new();
        match File::open(format!("states/{}", hex::encode(&txid))) {
            Err(e) => panic!("Cannot open file states/{} ",hex::encode(&txid)),
            Ok(mut r) => r.read_to_end(&mut buffer).unwrap()
        };
            

        let mut idx = 0;

        /* __vm_wait_for_call()
        -8     "li a7, 0xCBFB\n\t"
        +0     "ecall\n\t"
        +4     "li a0, 123\n\t"
        +8     "li a7, 93\n\t"
        +12    "ecall\n\t"
        +16     ...
        */
        // Restore PC, then increment past __vm_wait_for_call() asm
        let pc = VMSnapshot::<u64>::deserialize_reg(&mut buffer, &mut idx) + 16;
        println!("PC loaded: {:X}", &pc);
        machine.set_pc(Register::from_u64(pc));

        // Restore registers
        for (i, r) in machine.registers_mut().iter_mut().enumerate() {
            let ret = VMSnapshot::<u64>::deserialize_reg(&mut buffer[idx..], &mut idx);
            *r = Register::from_u64(ret);
            // println!("Loading reg {}: {:X}", i, *r);
        }

        // Restore pages
        for (i, p) in machine.memory_mut().pages.iter_mut().enumerate() {
            p.copy_from_slice(&buffer[idx..idx+RISCV_PAGESIZE]);
            idx += RISCV_PAGESIZE;
        }

        // Add hooks to detect PC miscalculation
        machine.registers_mut()[A0] = R::from_u64(222);
        machine.registers_mut()[A7] = R::from_u64(1111);

        // The recv addr is stored in A5, return that
        machine.registers()[A5]
    }

    pub fn has_saved_state(txid: &[u8]) -> bool {
        Path::new( &format!("states/{}", &hex::encode(txid))).exists()
    }

    pub fn save_to_file(&mut self, fname: String) {
        let mut buffer = match File::create(format!("states/{}", fname)) {
            Err(e) => panic!("Could not create file in VMSnapshot::save_to_file()"),
            Ok(f) => f,
        };
        let mut data: Vec<u8> = vec![];

        // Save PC
        self.serialize_reg(&self.pc, &mut data);
        println!("PC saved: {:X}", &self.pc.to_u64());

        // Serialize registers
        for (i, r) in self.registers.iter().enumerate() {
            self.serialize_reg(r, &mut data);
            // println!("Saving reg {}: {:X}", i, r.to_u64());
        }
        buffer.write(&data).unwrap();

        // Dump memory pages
        for p in self.pages.iter() {
            buffer.write(p).unwrap();
        }
    }
}
