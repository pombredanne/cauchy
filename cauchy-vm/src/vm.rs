use std::sync::Arc;

use bytes::Bytes;
use rand::rngs::ThreadRng;
use rand::RngCore;
use core::db::rocksdb::RocksDb;
use core::db::storing::*;
use core::db::*;
use futures::future::ok;
use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::sync::oneshot;
use futures::Async;
use std::fs::File;
use std::io::Write;

use ckb_vm::{
    CoreMachine, DefaultCoreMachine, DefaultMachineBuilder, Error, Memory, Register, SparseMemory,
    SupportMachine, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7, S1, S2,
};

use crate::performance::Performance;
use core::crypto::hashes::Identifiable;
use core::primitives::act::{Act, Message};
use core::primitives::transaction::Transaction;

pub struct VM {
    store: Arc<RocksDb>,
}

impl VM {
    pub fn new(store: Arc<RocksDb>) -> VM {
        VM { store }
    }

    pub fn run(
        &self,
        mailbox: Mailbox,
        tx: Transaction,
        parent_branch: oneshot::Sender<Performance>,
    ) -> Result<u8, Error> {
        // Construct session
        let mut performance = Performance::new();
        let id = tx.get_id();
        let session = Session {
            mailbox,
            id: id.clone(),
            timestamp: tx.get_time(),
            binary_hash: tx.get_binary_hash(),
            aux: tx.get_aux(),
            performance: &mut performance,
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

pub struct Session<'a> {
    mailbox: Mailbox,
    id: Bytes,
    timestamp: u64,
    binary_hash: Bytes,
    aux: Bytes,
    performance: &'a mut Performance,
    child_branch: Option<oneshot::Receiver<Performance>>,
    store: Arc<RocksDb>,
}

impl<'a> Session<'a> {
    fn recv(&mut self) -> Option<Message> {
        if let Some(branch) = self.child_branch.take() {
            let child_perforamnce = branch.wait().unwrap();
            *self.performance += child_perforamnce;
        }

        match self.mailbox.inbox.poll() {
            Ok(Async::Ready(msg)) => msg,
            _ => unreachable!(),
        }
    }

    fn send(&mut self, msg: Message) {
        if let Some(branch) = self.child_branch.take() {
            let child_perforamnce = branch.wait().unwrap();
            *self.performance += child_perforamnce;
        }

        let (child_send, child_branch) = oneshot::channel();
        self.child_branch = Some(child_branch);
        tokio::spawn(
            self.mailbox
                .outbox
                .clone()
                .send((msg, child_send))
                .map_err(|_| ())
                .and_then(|_| ok(())),
        );
    }

    fn put_store(&mut self, key: Bytes, value: Bytes) -> Result<(), failure::Error> {
        let result = self.store.put(&key, &value);
        self.performance.add_write(&self.id, key, value);
        result
    }

    fn get_store(&mut self, key: Bytes) -> Result<Option<Bytes>, failure::Error> {
        let value = self.store.get(&key);
        self.performance.add_read(&self.id, key);
        value
    }
}

impl<'a, Mac: SupportMachine> Syscalls<Mac> for Session<'a> {
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
                    let sender = msg.get_sender().to_vec();
                    machine
                        .memory_mut()
                        // TODO: Store at maximum the specified numbytes
                        .store_bytes(txid_addr as usize, &sender)
                        .unwrap();

                    // Store txid_sz
                    machine.set_register(S1, Mac::REG::from_usize(sender.len()));

                    // Store data received
                    let data = msg.get_payload().to_vec();
                    machine
                        .memory_mut()
                        // TODO: Store at maximum the specified numbytes
                        .store_bytes(data_addr as usize, &data)
                        .unwrap();

                    // Store data_sz
                    machine.set_register(S2, Mac::REG::from_usize(data.len()));
                // println!("Receiving message {:X?} of size {:?}", data, data.len());

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
                let mut key_bytes = self.id.to_vec();
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

                let result = self.put_store(Bytes::from(key_bytes), Bytes::from(value_bytes));

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
                let mut key_bytes = self.id.to_vec();
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
                let value = self.get_store(Bytes::from(key_bytes)).unwrap().unwrap();
                machine
                    .memory_mut()
                    // Store at maximum the specified numbytes
                    .store_bytes(buffer_addr as usize, &value[..buffer_sz])
                    .unwrap();

                Ok(true)
            }
            // __vm_auxdata(buff, size)
            0xCBFB => {
                let buffer_addr = machine.registers()[A5].to_usize();
                let buffer_sz = machine.registers()[A6].to_usize();

                // TODO: Limit to buffer_sz
                machine
                    .memory_mut()
                    .store_bytes(buffer_addr, &self.aux.to_vec())
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
            },
            // __vm_rand(buff, size)
            0xCBF9 => {
                let buffer_addr = machine.registers()[A5].to_usize();
                let buffer_sz = machine.registers()[A6].to_usize();
                let mut bytes : Vec::<u8> = vec![0;buffer_sz];
                ThreadRng::default().fill_bytes(&mut bytes);
                machine.memory_mut().store_bytes(buffer_addr, &bytes).unwrap();
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}