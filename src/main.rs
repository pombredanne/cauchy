extern crate rocksdb;
extern crate memcache;
extern crate bytes;
extern crate futures;

//use rocksdb::DB;

#[cfg(test)]
mod tests {
    mod test_script;
    mod test_varint;
}

pub mod db;

mod utils{
    pub mod serialisation;
}

pub mod primitives;

fn main() {
    // Initialise state database

    // Initialise UTXO database

    // Initialise stack database

    // Create a layered database

    // Init peers

    // 


}