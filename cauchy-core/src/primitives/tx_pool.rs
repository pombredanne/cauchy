use std::cmp::Ordering;
use std::collections::BinaryHeap;

use bytes::Bytes;
use failure::Error;
use itertools::Itertools;

use super::transaction::Transaction;

use crate::{
    crypto::hashes::Identifiable, utils::errors::TxPoolError, vm::performance::Performance,
};

#[derive(Clone, PartialEq, Eq)]
struct TxPoolItem {
    tx_id: Bytes,
    tx: Transaction,
    cached_perf: Option<Performance>,
}

impl From<Transaction> for TxPoolItem {
    fn from(tx: Transaction) -> TxPoolItem {
        TxPoolItem {
            tx_id: tx.get_id(),
            tx,
            cached_perf: None,
        }
    }
}

impl PartialOrd for TxPoolItem {
    fn partial_cmp(&self, other: &TxPoolItem) -> Option<Ordering> {
        match self.tx.get_time().partial_cmp(&other.tx.get_time()) {
            Some(Ordering::Equal) => self.tx_id.partial_cmp(&other.tx_id),
            Some(non_equal) => Some(non_equal),
            None => unreachable!(),
        }
    }
}

impl Ord for TxPoolItem {
    fn cmp(&self, other: &TxPoolItem) -> Ordering {
        match self.tx.get_time().cmp(&other.tx.get_time()) {
            Ordering::Equal => self.tx_id.cmp(&other.tx_id),
            other => other,
        }
    }
}

pub struct TxPool {
    txs: BinaryHeap<TxPoolItem>,
    size: usize,
}

impl TxPool {
    pub fn new(size: usize) -> TxPool {
        TxPool {
            txs: BinaryHeap::with_capacity(size),
            size,
        }
    }

    pub fn insert(&mut self, tx: Transaction, opt_tx_id: Option<Bytes>, cached_perf: Option<Performance>) -> Result<(), Error> {
        if self.txs.len() < self.size {
            let tx_id = match opt_tx_id {
                Some(tx_id) => tx_id,
                None => tx.get_id()
            };
            let item = TxPoolItem {
                tx_id,
                tx,
                cached_perf
            };
            self.txs.push(item);
            Ok(())
        } else {
            Err(TxPoolError::Full.into())
        }
    }

    pub fn to_sorted_txs(self) -> Vec<Transaction> {
        self.txs.into_sorted_vec().into_iter().map(|item| item.tx).collect()
    }

    pub fn insert_batch(
        &mut self,
        txs: Vec<Transaction>,
        validate_order: bool,
    ) -> Result<(), Error> {
        if self.txs.len() + txs.len() > self.size {
            return Err(TxPoolError::Full.into());
        }

        let last_item = match txs.last() {
            Some(some) => TxPoolItem::from(some.clone()),
            None => return Err(TxPoolError::EmptyInsert.into()),
        };

        let res: Result<Vec<TxPoolItem>, Error> = txs
            .into_iter()
            .map(|tx| TxPoolItem::from(tx))
            .tuple_windows()
            .map(|(item_a, item_b)| {
                // Check order
                if item_a < item_b || !validate_order {
                    Ok(item_a)
                } else {
                    return Err(TxPoolError::NotSorted.into());
                }
            })
            .collect();

        let items = res?;

        for item in items {
            self.txs.push(item);
        }
        self.txs.push(last_item);
        Ok(())
    }
}
