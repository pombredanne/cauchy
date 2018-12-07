extern crate bytes;
extern crate futures;
extern crate rocksdb;
extern crate blake2;


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
    // Initialise state database

    // Initialise TX pool database

    // Create a layered database

    // Init peers

    // 


}