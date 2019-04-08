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
}

impl<'a> VMSyscall<'a> {
    fn inbox_pop(&mut self, live_branch: Option<oneshot::Receiver<Act>>) -> Option<Message> {
        if let Some(branch) = live_branch {
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
        live_branch: Option<oneshot::Receiver<Act>>,
        msg: Message,
    ) -> oneshot::Receiver<Act> {
        if let Some(branch) = live_branch {
            let act = branch.wait().unwrap();
            self.act += act;
        }

        let (branch_send, branch_recv) = oneshot::channel();
        tokio::spawn(
            self.msg_sender
                .clone()
                .send((msg, branch_send))
                .map_err(|_| ())
                .and_then(|_| ok(())),
        );
        branch_recv
    }
}

impl<'a, Mac: SupportMachine> Syscalls<Mac> for VMSyscall<'a> {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, Error> {
        let code = &machine.registers()[A7];
        let code = code.to_i32();
        match code {
            //  __vm_reply(data, size)
            0xCBFF => {
                let sz = machine.registers()[A5].to_u32();
                let addr = machine.registers()[A6].to_u32();
                let mut msg_data = Vec::<u8>::new();
                for idx in addr..(addr + sz) {
                    msg_data.push(
                        machine
                            .memory_mut()
                            .load8(&Mac::REG::from_u32(idx))
                            .unwrap()
                            .to_u8(),
                    );
                }
                let msg = Message::new(
                    Bytes::from(&b"Sender addr"[..]),
                    Bytes::from(&b"Receiver addr"[..]),
                    Bytes::from(msg_data),
                );
                self.msg_send(Option::None, msg).poll().unwrap();
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
