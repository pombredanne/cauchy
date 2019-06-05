use rand::{rngs::ThreadRng, RngCore};
use std::fs::File;

use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use futures::future::{ok, Future};
use futures::sink::Sink;
use futures::stream::Stream;

use crate::crypto::hashes::Identifiable;
use crate::db::mongodb::MongoDB;
use crate::db::*;
use crate::primitives::act::Message;
use crate::primitives::transaction::Transaction;
use crate::vm::performance::Performance;
use crate::vm::{Mailbox, VM};
use bson::spec::BinarySubtype;
use bson::{bson, doc, *};
use futures::sync::{mpsc, oneshot};
use hex::*;
#[test]
fn test_simple() {
    let db = MongoDB::open_db("test_simple").unwrap();
    // let mut file = File::open("src/vm/tests/scripts/recv_then_sends_to_bob").unwrap();
    // let mut file = File::open("src/vm/tests/scripts/syscall").unwrap();
    let mut file = File::open("src/tests/vm/scripts/sha256").unwrap();
    // let mut file = File::open("src/vm/tests/scripts/ecdsa").unwrap();
    // let mut file = File::open("src/vm/tests/scripts/auxsend").unwrap();
    // let mut file = File::open("src/vm/tests/scripts_rust/target/riscv64gc-unknown-none-elf/release/scripts_rust").unwrap();
    let mut script = Vec::new();
    file.read_to_end(&mut script).unwrap();

    let payload = Bytes::from(&b"TestFunc Message"[..]);
    let msg = Message::new(
        Bytes::from(&b"TestFunc Sender addr"[..]),
        Bytes::from(&b"TestFunc Receiver addr"[..]),
        payload,
    );
    tokio::run({
        // Create inbox

        // Dummy terminator for root
        let (parent_branch, _) = oneshot::channel();

        // Init the VM
        let vm = VM::new(db);

        // Construct session
        let (outbox, outbox_recv) = mpsc::channel(512);
        let tx = Transaction::new(407548800, Bytes::from(&b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCD"[..]), Bytes::from(script));
        let (mailbox, inbox_send) = Mailbox::new(outbox);

        inbox_send
            .clone()
            .send(msg)
            .map_err(|_| ())
            .map(|_| ()) // Send a msg to inbox
            .and_then(move |_| {
                // Complete all spawned branches and print messages
                tokio::spawn(
                    tokio::timer::Delay::new(Instant::now() + Duration::from_secs(1))
                        .map_err(|_| ())
                        .and_then(|_| {
                            outbox_recv.for_each(|(msg, parent_branch)| {
                                parent_branch.send(()); // Complete branch
                                println!(
                                    "{:?} -- received msg -- {:?} from -- {:X?}",
                                    msg.get_receiver(),
                                    msg.get_payload(),
                                    msg.get_sender()
                                );
                                ok(())
                            })
                        }),
                );
                // Run the VM
                ok({
                    println!("Execution start");
                    let result = vm.run(
                        mailbox,
                        tx.clone(),
                        tx.get_id(),
                        Arc::new(Mutex::new(Performance::default())),
                        parent_branch,
                    );
                    assert!(result.is_ok());
                    assert_eq!(result.unwrap(), 0);
                    println!("Execution end");
                })
            })
    });
}

// #[test]
fn test_ecdsa() {
    let db = MongoDB::open_db("test_ecdsa").unwrap();
    // let mut file = File::open(
    //     "src/tests/scripts_rust/target/riscv64gc-unknown-none-elf/release/scripts_rust",
    // )
    // .unwrap();
    let mut file = File::open("src/tests/vm/scripts/ecdsa").unwrap();
    let mut script = Vec::new();
    file.read_to_end(&mut script).unwrap();

    let payload = Bytes::from(&b"Message"[..]);
    let msg = Message::new(
        Bytes::from(&b"Sender addr"[..]),
        Bytes::from(&b"Receiver addr"[..]),
        payload,
    );
    // Dummy terminator for root
    let (parent_branch, _) = oneshot::channel();

    // Init the VM
    let vm = VM::new(db);

    // Construct session
    let (outbox, outbox_recv) = mpsc::channel(512);
    let tx = Transaction::new(407548800, Bytes::from(&b"aux"[..]), Bytes::from(script));
    let (mailbox, inbox_send) = Mailbox::new(outbox);
    let result = vm.run(
        mailbox,
        tx.clone(),
        tx.get_id(),
        Arc::new(Mutex::new(Performance::default())),
        parent_branch,
    );
    assert_eq!(result.unwrap(), 8);
}

#[test]
fn test_store() {
    let db = MongoDB::open_db("test_store").unwrap();
    // db.dropall(&DataType::State);
    let mut file = File::open("src/tests/vm/scripts/basic_store").unwrap();
    let mut script = Vec::new();
    file.read_to_end(&mut script).unwrap();

    let payload = Bytes::from(&b"Message"[..]);
    let msg = Message::new(
        Bytes::from(&b"Sender addr"[..]),
        Bytes::from(&b"Receiver addr"[..]),
        payload,
    );
    // Dummy terminator for root
    let (parent_branch, _) = oneshot::channel();

    // Init the VM
    let vm = VM::new(db);

    // Construct session
    let (outbox, outbox_recv) = mpsc::channel(512);
    let tx = Transaction::new(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        Bytes::from(&b"aux"[..]),
        Bytes::from(script),
    );
    let (mailbox, inbox_send) = Mailbox::new(outbox);

    let result = vm.run(
        mailbox,
        tx.clone(),
        tx.get_id(),
        Arc::new(Mutex::new(Performance::default())),
        parent_branch,
    );
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_contract() {
    let db = MongoDB::open_db("test_contract").unwrap();
    let mut file = File::open("src/tests/vm/scripts/contracts/gas").unwrap();
    let mut script = Vec::new();
    file.read_to_end(&mut script).unwrap();

    // let payload = Bytes::from(hex::decode("020010000000000000000000000000000000000000000000000000000000000000000000").unwrap());
    //                                     |   |
    let payload = Bytes::from(hex::decode("01004141414141414141414141414141414141414141414141414141414141414141CC6758ED4C6DCB94A366FCDF56FE9F2BDB99402AF9FFA3B60FD44407B3E71B82A74BD94B07B89CC12DFAC246908877B6D636975A34068D7A7B9E00AEF07B5BAF0A000000000000009A93AF48B52735712B0BCF0FFFFE469642268C7C1AFA1A21C81C97D3337E0BD3899EB7DA0EEC13F11DFD629D04D67798D3A843E286179D34F851645BD377D5A5").unwrap());
    let mut bytes: Vec<u8> = vec![0; 32];
    ThreadRng::default().fill_bytes(&mut bytes);
    let msg = Message::new(
        // Bytes::from(&b"TestFunc Sender addr"[..]),
        Bytes::from(bytes),
        Bytes::from(&b"TestFunc Receiver addr"[..]),
        payload,
    );
    tokio::run({
        // Create inbox

        // Dummy terminator for root
        let (parent_branch, _) = oneshot::channel();

        // Init the VM
        let vm = VM::new(db.clone());

        // Construct session
        let (outbox, outbox_recv) = mpsc::channel(512);
        let tx = Transaction::new(407548800, Bytes::from(hex::decode("94fe2b6dee4e447b4e1ce75d4a3ad3bbbd095979866cea1613e72fc967a73281E803000000000000").unwrap()), Bytes::from(script));
        let (mailbox, inbox_send) = Mailbox::new(outbox);

        db.update(
            &DataType::State,
            doc! {"p" : Bson::Binary(BinarySubtype::Generic, tx.get_id().to_vec())},
            doc! { "$unset" : {"p" : ""} },
        )
        .unwrap();

        inbox_send
            .clone()
            .send(msg)
            .map_err(|_| ())
            .map(|_| ()) // Send a msg to inbox
            .and_then(move |_| {
                // Complete all spawned branches and print messages
                tokio::spawn(
                    tokio::timer::Delay::new(Instant::now() + Duration::from_secs(1))
                        .map_err(|_| ())
                        .and_then(|_| {
                            outbox_recv.for_each(|(msg, parent_branch)| {
                                parent_branch.send(()); // Complete branch
                                println!(
                                    "{:?} -- received msg -- {:?} from -- {:X?}",
                                    msg.get_receiver(),
                                    msg.get_payload(),
                                    msg.get_sender()
                                );
                                ok(())
                            })
                        }),
                );
                // Run the VM
                ok({
                    println!("Execution start");
                    let result = vm.run(
                        mailbox,
                        tx.clone(),
                        tx.get_id(),
                        Arc::new(Mutex::new(Performance::default())),
                        parent_branch,
                    );
                    assert!(result.is_ok());
                    assert_eq!(result.unwrap(), 0);
                    println!("Execution end");
                })
            })
    });
}
