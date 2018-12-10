extern crate bytes;
extern crate rocksdb;
extern crate blake2;

use crypto::hashes::oddsketch::*;
use primitives::work_site::*;
use bytes::Bytes;

#[cfg(test)]
mod tests {
    mod test_varint;
    mod test_db;
    mod test_transaction;
    mod test_hash;
    mod test_byte_tools;
}

pub mod db;

pub mod utils;

mod crypto {
    pub mod hashes;
}

pub mod primitives;

fn main() {
    // Mining
    let state_one = vec![Bytes::from(&b"a"[..]), Bytes::from(&b"b"[..]), Bytes::from(&b"c"[..]), Bytes::from(&b"d"[..]), Bytes::from(&b"e"[..])];
    let state_two = vec![Bytes::from(&b"a"[..]), Bytes::from(&b"b"[..]), Bytes::from(&b"c"[..]), Bytes::from(&b"d"[..])];
    let state_three = vec![Bytes::from(&b"far"[..]), Bytes::from(&b"away"[..]), Bytes::from(&b"state"[..])];

    let sketch_one = state_one.odd_sketch();
    let sketch_two = state_two.odd_sketch();
    let sketch_three = state_three.odd_sketch();


    let pk = Bytes::from(&b"\x01\x01\x01\x01\x01\x01"[..]);
    let mut worksite = WorkSite::init(pk);

    let mut best: u32 = 512;
    let mut size: u32;

    let mut i = 0;
    println!("START MINING");
    loop {
        size = worksite.mine(&sketch_one);
        if size < best {
            best = size;
            println!("\nNew state found!");
            println!("{} tries since last!", i);
            println!("Distance to state {}", size);
            println!("Distance to nearby state {}", worksite.mine(&sketch_two));
            println!("Distance to distant state {}", worksite.mine(&sketch_three));
        }

        worksite.increment();
        i += 1;
    }

}