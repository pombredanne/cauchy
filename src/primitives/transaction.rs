use primitives::script::Script;

/* 
             v Length of next script
VarInt || VarInt || Script || VarInt || Script || ... || VarInt || Script
  ^ Encoded PassBy's             ^ Length of next script

-First script is executed and must return true, the others are added to the "library".
-Number of scripts should be bounded at 512 (as VarInt size is bounded at 64 bytes)
*/

// Structure of Transaction
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction(Vec<Script>);

impl Transaction {
    pub fn new(scripts: Vec<Script>) -> Self {
        Transaction(scripts)
    }

    pub fn get_script(&self, i: usize) -> Result<&Script, String> {
        if self.0.len() < i {
            Err("Script out of range".to_string())
        } else {
            Ok(&self.0[i])
        }
    }

    pub fn get_len(&self) -> usize {
        self.0.len()
    }
}

impl From<Transaction> for Vec<Script> {
    fn from(tx: Transaction) -> Vec<Script> {
        tx.0
    }
}