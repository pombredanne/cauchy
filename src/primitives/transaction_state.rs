use std::collections::hash_set::Iter;
use std::collections::HashSet;

/*
Wraps a set containing unspent outputs
*/

pub struct TransactionState(HashSet<u32>);

impl TransactionState {
	pub fn init(n_spendable: u32) -> TransactionState {
		let set: HashSet<u32> = (0..n_spendable).collect(); 
		TransactionState(set)
	}

	pub fn new(set: HashSet<u32>) -> TransactionState {
		TransactionState(set)
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