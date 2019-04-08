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

use ckb_vm::{
    CoreMachine, DefaultCoreMachine, DefaultMachineBuilder, Error, Memory, Register, SparseMemory,
    SupportMachine, Syscalls, A0, A1, A2, A3, A4, A5, A6, A7,
};

use core::primitives::act::{Act, Message};

pub struct VM {
    timestamp: u64,
    binary: Bytes,
    act: Act,
    msg_sender: Sender<(Message, oneshot::Sender<Act>)>,
    inbox: Receiver<Message>,
    store: Arc<RocksDb>,
    terminate_branch: oneshot::Sender<Act>,
}

impl VM {
    pub fn new(
        timestamp: u64,
        binary: Bytes,
        msg_sender: Sender<(Message, oneshot::Sender<Act>)>,
        terminate_branch: oneshot::Sender<Act>,
        store: Arc<RocksDb>,
    ) -> (VM, Sender<Message>) {
        let (inbox_send, inbox) = channel(128);
        println!("Got here");
        (
            VM {
                act: Act::new(),
                timestamp,
                binary,
                msg_sender,
                inbox,
                store,
                terminate_branch,
            },
            inbox_send,
        )
    }

    pub fn terminate(self) {
        self.terminate_branch.send(self.act);
    }

    pub fn run(&mut self) -> Result<u8, Error> {
        let mut machine =
            DefaultMachineBuilder::<DefaultCoreMachine<u64, SparseMemory<u64>>>::default()
                .syscall(Box::new(VMSyscall {
                    act: self.act.clone(),
                    msg_sender: self.msg_sender.clone(),
                    inbox: self.inbox.by_ref(),
                    live_branch: None
                }))
                .build();
        machine = machine
            .load_program(&self.binary[..], &vec![b"syscall".to_vec()])
            .unwrap();
        let result = machine.interpret();
        result
    }
}

pub struct VMSyscall<'a> {
    act: Act,
    msg_sender: Sender<(Message, oneshot::Sender<Act>)>,
    inbox: &'a mut Receiver<Message>,
    live_branch: Option<oneshot::Receiver<Act>>
}

impl<'a> VMSyscall<'a> {
    fn inbox_pop(&mut self) -> Option<Message> {
        if let Some(branch) = self.live_branch.take() {
            let act = branch.wait().unwrap();
            self.act += act;
        }

        match self.inbox.poll() {
            Ok(Async::Ready(msg)) => msg,
            _ => unreachable!(),
        }
    }

    fn msg_send(
        &mut self,
        msg: Message,
    ) {
        if let Some(branch) = self.live_branch.take() {
            let act = branch.wait().unwrap();
            self.act += act;
        }

        let (branch_send, branch_recv) = oneshot::channel();
        self.live_branch = Some(branch_recv);
        tokio::spawn(
            self.msg_sender
                .clone()
                .send((msg, branch_send))
                .map_err(|_| ())
                .and_then(|_| ok(())),
        );
    }
}

impl<'a, Mac: SupportMachine> Syscalls<Mac> for VMSyscall<'a> {
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
                    Bytes::from(&b"Misc Sender data_addr"[..]),
                    Bytes::from(txid_bytes),
                    Bytes::from(data_bytes),
                );
                self.msg_send(msg).poll().unwrap();
                Ok(true)
            }
            // void __vm_recv(txid, txid_sz, data, data_sz)
            0xCBFE => {
                if let Some(msg) = self.inbox_pop() {
                    let txid_addr = machine.registers()[A3].to_u64();
                    let txid_sz_addr = machine.registers()[A4].to_u64();
                    let data_addr = machine.registers()[A5].to_u64();
                    let data_sz_addr = machine.registers()[A6].to_u64();

                    // Store txid
                    let sender = msg.get_sender().to_vec();
                    machine.memory_mut().store_bytes(txid_addr as usize, &sender).unwrap();

                    // Store txid_sz
                    let s = sender.len() as u64;
                     machine.memory_mut().store_bytes(
                         txid_sz_addr as usize,
                         &vec![s as u8, (s >> 8) as u8, (s >> 16) as u8, (s >> 24) as u8],//, (s >> 32) as u8,(s >> 40) as u8, (s >> 48) as u8, (s >> 56) as u8]
                     ).unwrap();

                    // Store data received
                    let data = msg.get_payload().to_vec();
                    machine.memory_mut().store_bytes(data_addr as usize, &data).unwrap();

                    // Store data_sz
                    let s = data.len() as u64;
                     machine.memory_mut().store_bytes(
                         data_sz_addr as usize,
                         &vec![s as u8, (s >> 8) as u8, (s >> 16) as u8, (s >> 24) as u8,]// (s >> 32) as u8,(s >> 40) as u8, (s >> 48) as u8, (s >> 56) as u8]
                     ).unwrap();

                }
                else
                {

                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
