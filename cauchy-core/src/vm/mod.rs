pub mod performance;
pub mod session;

#[macro_use(bson, doc)]
use std::sync::{Arc, Mutex};
use bytes::*;

use futures::sync::{
    mpsc::{channel, Receiver, Sender},
    oneshot,
};
use rand::{rngs::ThreadRng, RngCore};

use performance::Performance;
use session::Session;

use crate::{
    crypto::hashes::Identifiable,
    db::{mongodb::MongoDB, storing::*, *},
    primitives::{
        act::{Act, Message},
        transaction::Transaction,
    },
};
use ckb_vm::{
    CoreMachine, DefaultCoreMachine, DefaultMachineBuilder, Error, Memory, Register, SparseMemory,
    SupportMachine, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7, S1, S2,
};
use std::io::{Read, Write};

pub struct VM {
    store: MongoDB,
}

impl VM {
    pub fn new(store: MongoDB) -> VM {
        VM { store }
    }

    pub fn run(
        &self,
        mailbox: Mailbox,
        tx: Transaction,
        perfid: Bytes,
        parent_branch: oneshot::Sender<Arc<Mutex<Performance>>>,
    ) -> Result<u8, Error> {
        // Construct session
        let mut performance = Arc::new(Mutex::new(Performance::new()));
        let id = tx.get_id();
        let session = Session {
            mailbox,
            id: id.clone(),
            perfid: perfid,
            timestamp: tx.get_time(),
            binary_hash: tx.get_binary_hash(),
            aux: tx.get_aux(),
            performance: performance.clone(),
            child_branch: None,
            store: self.store.clone(),
        };
        // Init machine
        let mut machine =
            DefaultMachineBuilder::<DefaultCoreMachine<u64, SparseMemory<u64>>>::default()
                .syscall(Box::new(session))
                .build();

        // Execute binary
        machine = machine
            .load_program(&tx.get_binary(), &vec![b"syscall".to_vec()])
            .unwrap();
        let result = machine.interpret();
        drop(machine);

        // Send termination alert to parent
        parent_branch.send(performance);

        // Return act and result
        result
    }
}

pub struct Mailbox {
    inbox: Receiver<Message>,
    outbox: Sender<(Message, oneshot::Sender<Performance>)>,
}

impl Mailbox {
    pub fn new(
        outbox: Sender<(Message, oneshot::Sender<Performance>)>,
    ) -> (Mailbox, Sender<Message>) {
        let (inbox_send, inbox) = channel(128);
        (Mailbox { inbox, outbox }, inbox_send)
    }
}

impl<'a, Mac: SupportMachine> Syscalls<Mac> for Session {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, Error> {
        let code = &machine.registers()[A7];
        let code = code.to_i32();

        // fn read_from_addr<Mac>(addr: u32, size: u32, machine: &Mac) -> Vec<u8> {
        //     let bytes = Vec::<u8>::new();
        //     for idx in addr..(addr + size) {
        //         bytes.push(
        //             machine
        //                 .memory()
        //                 .load8(&Mac::REG::from_u64(idx))
        //                 .unwrap()
        //                 .to_u8(),
        //         );
        //     }
        //     bytes
        // }

        match code {
            //  __vm_send(txid, txid_size, data, size)
            0xCBFF => {
                let txid_addr = machine.registers()[A3].to_u64();
                let txid_sz = machine.registers()[A4].to_u64();
                let data_addr = machine.registers()[A5].to_u64();
                let data_sz = machine.registers()[A6].to_u64();

                // Load txid
                let mut txid_bytes = Vec::<u8>::new();
                for idx in txid_addr..(txid_addr + txid_sz) {
                    txid_bytes.push(
                        machine
                            .memory_mut()
                            .load8(&Mac::REG::from_u64(idx))
                            .unwrap()
                            .to_u8(),
                    );
                }

                // Load data to be sent
                let mut data_bytes = Vec::<u8>::new();
                for idx in data_addr..(data_addr + data_sz) {
                    data_bytes.push(
                        machine
                            .memory_mut()
                            .load8(&Mac::REG::from_u64(idx))
                            .unwrap()
                            .to_u8(),
                    );
                }

                let msg = Message::new(
                    // Bytes::from(&b"__vm_send()"[..]),
                    self.id.clone(),
                    Bytes::from(txid_bytes),
                    Bytes::from(data_bytes.clone()),
                );
                self.send(msg);
                // println!("Sending message of size {:}", data_sz);
                // if data_sz < 200 {
                //     self.send(msg);
                // } else {
                //     assert!(false);
                // }
                Ok(true)
            }
            // void __vm_recv(txid, txid_sz, data, data_sz)
            0xCBFE => {
                if let Some(msg) = self.recv() {
                    let txid_addr = machine.registers()[A3].to_u64();
                    let txid_sz_addr = machine.registers()[A4].to_usize();
                    let data_addr = machine.registers()[A5].to_u64();
                    let data_sz_addr = machine.registers()[A6].to_usize();

                    // Store txid
                    if (txid_addr != 0) && (txid_sz_addr != 0) {
                        let sender = msg.get_sender().to_vec();
                        machine
                            .memory_mut()
                            // TODO: Store at maximum the specified numbytes
                            .store_bytes(txid_addr as usize, &sender)
                            .unwrap();

                        // Store txid_sz
                        machine.set_register(S1, Mac::REG::from_usize(sender.len()));
                    }
                    // Store data received
                    if (data_addr != 0) && (data_sz_addr != 0) {
                        let data = msg.get_payload().to_vec();
                        machine
                            .memory_mut()
                            // TODO: Store at maximum the specified numbytes
                            .store_bytes(data_addr as usize, &data)
                            .unwrap();

                        // Store data_sz
                        machine.set_register(S2, Mac::REG::from_usize(data.len()));
                    }

                // Dump memory to file
                // let mut file = File::create("./memdump.bin").unwrap();
                // let mut i = 0;
                // while {
                //     match machine.memory_mut().load8(&Mac::REG::from_u32(i)) {
                //         Ok(v) => {
                //             file.write(&[v.to_u8()]).unwrap();
                //             true
                //         }
                //         _ => (false),
                //     }
                // } {
                //     i += 1;
                // }
                } else {
                    machine.set_register(S1, Mac::REG::zero());
                    machine.set_register(S2, Mac::REG::zero());
                }
                Ok(true)
            }
            // void __vm_store(key, value)
            0xCBFD => {
                let key_addr = machine.registers()[A3].to_u64();
                let key_sz = machine.registers()[A4].to_u64();
                let value_addr = machine.registers()[A5].to_u64();
                let value_sz = machine.registers()[A6].to_u64();

                // Load key
                let mut key_bytes = Vec::<u8>::new();
                for idx in key_addr..(key_addr + key_sz) {
                    key_bytes.push(
                        machine
                            .memory_mut()
                            .load8(&Mac::REG::from_u64(idx))
                            .unwrap()
                            .to_u8(),
                    );
                }
                let mut value_bytes = Vec::<u8>::new();
                for idx in value_addr..(value_addr + value_sz) {
                    value_bytes.push(
                        machine
                            .memory_mut()
                            .load8(&Mac::REG::from_u64(idx))
                            .unwrap()
                            .to_u8(),
                    );
                }

                let result =
                    ValueStore(Bytes::from(key_bytes)).to_db(self, Some(Bytes::from(value_bytes)));

                // TODO: use as return value to __vm_store()
                assert!(result.is_ok());

                Ok(true)
            }
            // void __vm_lookup(key, *value)
            0xCBFC => {
                let key_addr = machine.registers()[A3].to_u64();
                let key_sz = machine.registers()[A4].to_u64();
                let buffer_addr = machine.registers()[A5].to_u64();
                let buffer_sz = machine.registers()[A6].to_usize();

                // Load the key
                let mut key_bytes = Vec::<u8>::new();
                for idx in key_addr..(key_addr + key_sz) {
                    key_bytes.push(
                        machine
                            .memory_mut()
                            .load8(&Mac::REG::from_u64(idx))
                            .unwrap()
                            .to_u8(),
                    );
                }

                // TODO: Do something useful on error
                match ValueStore::from_db(self, Bytes::from(key_bytes)) {
                    Ok(Some(some)) => {
                        machine
                            .memory_mut()
                            // Store at maximum the specified numbytes
                            .store_bytes(buffer_addr as usize, &some.0[..buffer_sz])
                            .unwrap();
                    }
                    Ok(None) => (println!("Key not found")),
                    Err(e) => println!("{:?}", e),
                }
                Ok(true)
            }
            // __vm_auxdata(buff, size)
            0xCBFB => {
                let addr = machine.registers()[A4].to_usize();
                let index = machine.registers()[A5].to_usize();
                let size = machine.registers()[A6].to_usize();

                // Cap size at length of auxdata
                let size = if size > self.aux.len() {
                    self.aux.len()
                } else {
                    size
                };

                // TODO: Limit to buffer_sz
                machine
                    .memory_mut()
                    .store_bytes(addr, &self.aux.to_vec()[index..(index + size)])
                    .unwrap();
                machine.set_register(S2, Mac::REG::from_usize(self.aux.len()));

                Ok(true)
            }
            // __vm_sendfromaux(txidsz, datasz)
            0xCBFA => {
                let txid_sz = machine.registers()[A5].to_usize();
                let data_sz = machine.registers()[A6].to_usize();
                let msg = Message::new(
                    self.id.clone(),
                    Bytes::from(&self.aux[..txid_sz]),
                    Bytes::from(&self.aux[txid_sz + 1..txid_sz + data_sz]),
                );
                self.send(msg);
                Ok(true)
            }
            // __vm_rand(buff, size)
            0xCBF9 => {
                let buffer_addr = machine.registers()[A5].to_usize();
                let buffer_sz = machine.registers()[A6].to_usize();
                let mut bytes: Vec<u8> = vec![0; buffer_sz];
                ThreadRng::default().fill_bytes(&mut bytes);
                machine
                    .memory_mut()
                    .store_bytes(buffer_addr, &bytes)
                    .unwrap();
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
