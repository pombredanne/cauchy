use std::collections::hash_set::Iter;
use std::collections::HashSet;

/*
Wraps a set containing unspent outputs
*/

pub struct SpendState(HashSet<u32>);

impl SpendState {
    pub fn init(n_spendable: u32) -> SpendState {
        let set: HashSet<u32> = (0..n_spendable).collect();
        SpendState(set)
    }

    pub fn new(set: HashSet<u32>) -> SpendState {
        SpendState(set)
    }

    pub fn iter(&self) -> Iter<u32> {
        self.0.iter()
    }

    pub fn spend(&mut self, id: u32) -> Result<(), String> {
        if self.0.contains(&id) {
            self.0.remove(&id);
            Ok(())
        } else {
            Err("Already spent".to_string())
        }
    }

    pub fn unspend(&mut self, id: u32) -> Result<(), String> {
        if self.0.contains(&id) {
            Err("Already unspent".to_string())
        } else {
            self.0.insert(id);
            Ok(())
        }
    }
}
