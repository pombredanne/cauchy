use primitives::script::Script;

/* 
                      v Length of next script
VarInt || VarInt || VarInt || Script || VarInt || Script || ... || VarInt || Script
  ^UTC      ^ Encoded PassBy's             ^ Length of next script

-First script is executed and must return true, the others are added to the "library".
-Number of scripts should be bounded at 512 (as VarInt size is bounded at 64 bytes)
*/

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction(u32, Vec<Script>);

impl Transaction {
    pub fn new(time: u32, scripts: Vec<Script>) -> Self {
        Transaction(time, scripts)
    }

    pub fn get_script(&self, i: usize) -> Result<&Script, String> {
        if self.1.len() < i {
            Err("Script out of range".to_string())
        } else {
            Ok(&self.1[i])
        }
    }

    pub fn len(&self) -> usize {
        self.1.len()
    }

    pub fn time(&self) -> u32 {
        self.0
    }
}

impl From<Transaction> for Vec<Script> {
    fn from(tx: Transaction) -> Vec<Script> {
        tx.1
    }
}