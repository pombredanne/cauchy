extern crate blake2;
extern crate bytes;
extern crate rand;
extern crate rocksdb;
extern crate secp256k1;

use bytes::Bytes;
use crypto::hashes::oddsketch::*;
use primitives::work_site::*;
use std::time::SystemTime;

#[cfg(test)]
mod test {
    mod byte_op_tests;
    mod db_tests;
    mod hash_tests;
    mod signature_tests;
    mod transaction_state_tests;
    mod transaction_tests;
    mod varint_tests;
}

pub mod consensus;
pub mod db;
pub mod primitives;
pub mod utils;
mod crypto;


fn main() {
    // Mining
    let state_one = vec![
        Bytes::from(&b"a"[..]),
        Bytes::from(&b"b"[..]),
        Bytes::from(&b"c"[..]),
        Bytes::from(&b"d"[..]),
        Bytes::from(&b"e"[..]),
        Bytes::from(&b"f"[..]),
    ];
    let state_two = vec![
        Bytes::from(&b"a"[..]),
        Bytes::from(&b"b"[..]),
        Bytes::from(&b"c"[..]),
        Bytes::from(&b"d"[..]),
        Bytes::from(&b"e"[..]),
    ];
    let state_three = vec![
        Bytes::from(&b"far"[..]),
        Bytes::from(&b"away"[..]),
        Bytes::from(&b"state"[..]),
    ];

    let sketch_one = state_one.odd_sketch();
    let sketch_two = state_two.odd_sketch();
    let sketch_three = state_three.odd_sketch();

    let pk = Bytes::from(&b"\x01\x01\x01\x01\x01\x01"[..]);
    let worksite = WorkSite::init(pk);

    let mut best: u32 = 512;
    let mut size: u32;

    let mut now = SystemTime::now();
    let mut i = 0;

    println!("START MINING");
    loop {
        size = worksite.mine(&sketch_one);
        if size < best {
            best = size;
            println!("\nNew best found!");
            println!(
                "{} seconds since last discovery",
                now.elapsed().unwrap().as_secs()
            );
            println!("{} hashes since last discovery", i);
            println!("Distance to state {}", size);
            println!("Distance to nearby state {}", worksite.mine(&sketch_two));
            println!("Distance to distant state {}", worksite.mine(&sketch_three));
            i = 0;
            now = SystemTime::now();
        }

        worksite.increment();
        i += 1;
    }
}
