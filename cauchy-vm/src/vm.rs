use std::sync::Arc;

use bytes::Bytes;
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
    SupportMachine, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7, S1, S2
};

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
        parent_branch: oneshot::Sender<()>,
    ) -> (Act, Result<u8, Error>) {
        // Construct session
        let mut act = Act::new();
        let session = Session {
            mailbox,
            id: tx.get_id(),
            timestamp: tx.get_time(),
            binary_hash: tx.get_binary_hash(),
            act: &mut act,
            child_branch: None,
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
        parent_branch.send(());

        // Return act and result
        (act, result)
    }
}

pub struct Mailbox {
    inbox: Receiver<Message>,
    outbox: Sender<(Message, oneshot::Sender<()>)>,
}

impl Mailbox {
    pub fn new(outbox: Sender<(Message, oneshot::Sender<()>)>) -> (Mailbox, Sender<Message>) {
        let (inbox_send, inbox) = channel(128);
        (Mailbox { inbox, outbox }, inbox_send)
    }
}

pub struct Session<'a> {
    mailbox: Mailbox,
    id: Bytes,
    timestamp: u64,
    binary_hash: Bytes,
    act: &'a mut Act,
    child_branch: Option<oneshot::Receiver<()>>,
}

impl<'a> Session<'a> {
    fn recv(&mut self) -> Option<Message> {
        if let Some(branch) = self.child_branch.take() {
            branch.wait().unwrap();
        }

        match self.mailbox.inbox.poll() {
            Ok(Async::Ready(msg)) => msg,
            _ => unreachable!(),
        }
    }

    fn send(&mut self, msg: Message) {
        if let Some(branch) = self.child_branch.take() {
            branch.wait().unwrap();
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
                    //self.id.clone(),
                    Bytes::from(&b"__vm_send() sender addr"[..]),
                    Bytes::from(txid_bytes),
                    Bytes::from(data_bytes.clone()),
                );
                println!("Sending message of size {:}", data_sz);
                if (data_sz < 200) {
                    self.send(msg);
                }
                else {
                    assert!(false);
                }
                Ok(true)
            }
            // void __vm_recv(txid, txid_sz, data, data_sz)
            0xCBFE => {
                if let Some(msg) = self.recv() {
                    let txid_addr = machine.registers()[A3].to_u64();
                    let txid_sz_addr = machine.registers()[A4].to_u64();
                    let data_addr = machine.registers()[A5].to_u64();
                    let data_sz_addr = machine.registers()[A6].to_u64();

                    // Store txid
                    let sender = msg.get_sender().to_vec();
                    machine
                        .memory_mut()
                        .store_bytes(txid_addr as usize, &sender)
                        .unwrap();

                    // Store txid_sz
                    machine.set_register(S1, Mac::REG::from_usize(sender.len()));

                    // Store data received
                    let data = msg.get_payload().to_vec();
                    machine
                        .memory_mut()
                        .store_bytes(data_addr as usize, &data)
                        .unwrap();

                    // Store data_sz
                    machine.set_register(S2, Mac::REG::from_usize(data.len()));
                    println!("Receiving message {:X?} of size {:?}", data, data.len());

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
            _ => Ok(false),
        }
    }
}
