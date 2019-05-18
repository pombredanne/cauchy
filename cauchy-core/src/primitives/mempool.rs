use std::cmp::Ordering;
use std::collections::BinaryHeap;

use bytes::Bytes;
use failure::Error;
use itertools::Itertools;

use super::transaction::Transaction;

use crate::{crypto::hashes::Identifiable, vm::performance::Performance, utils::errors::MempoolError};

#[derive(Clone, PartialEq, Eq)]
struct MempoolItem {
    tx_id: Bytes,
    tx: Transaction,
    cached_perf: Option<Performance>,
}

impl From<Transaction> for MempoolItem {
    fn from(tx: Transaction) -> MempoolItem {
        MempoolItem {
            tx_id: tx.get_id(),
            tx,
            cached_perf: None,
        }
    }
}

impl PartialOrd for MempoolItem {
    fn partial_cmp(&self, other: &MempoolItem) -> Option<Ordering> {
        match self.tx.get_time().partial_cmp(&other.tx.get_time()) {
            Some(Ordering::Equal) => self.tx_id.partial_cmp(&other.tx_id),
            Some(non_equal) => Some(non_equal),
            None => unreachable!(),
        }
    }
}

impl Ord for MempoolItem {
    fn cmp(&self, other: &MempoolItem) -> Ordering {
        match self.tx.get_time().cmp(&other.tx.get_time()) {
            Ordering::Equal => self.tx_id.cmp(&other.tx_id),
            other => other,
        }
    }
}

pub struct Mempool {
    txs: BinaryHeap<MempoolItem>,
    size: usize,
}

impl Mempool {
    pub fn new() -> Mempool {
        let mempool_size = 1024; // TODO: Add to configurable constants
        Mempool {
            txs: BinaryHeap::with_capacity(mempool_size),
            size: mempool_size,
        }
    }

    // pub fn insert(&mut self, tx: Transaction, opt_tx_id: Option<Bytes>, cached_perf: Option<Performance>) {
    //     if self.txs.len() < self.size {
    //         let tx_id = match opt_tx_id {
    //             Some(tx_id) => tx_id,
    //             None => tx.get_id()
    //         };
    //         let item = MempoolItem {
    //             tx_id,
    //             tx,
    //             cached_perf
    //         };
    //         self.txs.push()
    //     }
    // }

    pub fn insert_batch(&mut self, txs: Vec<Transaction>, validate_order: bool) -> Result<(), Error> {
        if self.txs.len() + txs.len() > self.size {
            return Err(MempoolError::MempoolFull.into());
        }

        let last_item = match txs.last() {
            Some(some) => MempoolItem::from(some.clone()),
            None => return Err(MempoolError::MempoolFull.into()),
        };

        let res: Result<Vec<MempoolItem>, Error> = txs
            .into_iter()
            .map(|tx| MempoolItem::from(tx))
            .tuple_windows()
            .map(|(item_a, item_b)| {
                // Check order
                if item_a < item_b || !validate_order {
                    Ok(item_a)
                } else {
                    return Err(MempoolError::NotSorted.into());
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
