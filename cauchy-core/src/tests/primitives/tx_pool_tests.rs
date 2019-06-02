use bytes::Bytes;
use rand::Rng;

use crate::{
    primitives::{transaction::Transaction, tx_pool::TxPool},
    utils::errors::TxPoolError,
};

fn generate_random_tx(time: u64) -> Transaction {
    let mut rng = rand::thread_rng();
    let aux_data: [u8; 8] = rng.gen();
    let binary: [u8; 8] = rng.gen();
    Transaction::new(time, Bytes::from(&aux_data[..]), Bytes::from(&binary[..]))
}

#[test]
fn test_put_sorted() {
    let tx_a = generate_random_tx(0);
    let tx_b = generate_random_tx(1);
    let tx_c = generate_random_tx(2);

    let mut mempool = TxPool::with_capacity(3);

    assert!(mempool.insert_batch(vec![tx_a, tx_b, tx_c], true).is_ok())
}

#[test]
fn test_put_full() {
    let tx_a = generate_random_tx(0);
    let tx_b = generate_random_tx(1);
    let tx_c = generate_random_tx(2);
    let tx_d = generate_random_tx(3);

    let mut mempool = TxPool::with_capacity(3);

    assert!(mempool
        .insert_batch(vec![tx_a, tx_b, tx_c, tx_d], true)
        .is_err())
}

#[test]
fn test_put_unsorted() {
    let tx_a = generate_random_tx(0);
    let tx_b = generate_random_tx(1);
    let tx_c = generate_random_tx(2);

    let mut mempool = TxPool::with_capacity(3);

    assert!(mempool.insert_batch(vec![tx_a, tx_c, tx_b], true).is_err())
}
