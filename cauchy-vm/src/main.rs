extern crate ckb_vm;

pub mod vm;

use core::db::rocksdb::RocksDb;
use core::db::storing::*;
use core::db::*;
use vm::VM;
use bytes::Bytes;
use std::sync::Arc;

fn main() {
    println!("Hello, world!");

    let tx_db = RocksDb::open_db(".cauchy/tests/db_vm_test_syscall2/").unwrap();
    let script = Bytes::from(&b"Hello World!"[..]);
    let msg = Bytes::from(&b"Message"[..]);
    let vm_test = VM::new(script, msg, 0, Arc::new(tx_db) );
}

#[cfg(test)]
mod tests {
    mod test_simple;
}