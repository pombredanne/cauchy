extern crate bytes;
extern crate futures;
extern crate rocksdb;


#[cfg(test)]
mod tests {
    mod test_script;
    mod test_varint;
    mod test_db;
    mod test_transaction;
}

pub mod db;

mod utils{
    pub mod serialisation;
    pub mod hash;
}

pub mod primitives;

fn main() {
    // Initialise state database

    // Initialise TX pool database

    // Create a layered database

    // Init peers

    // 


}