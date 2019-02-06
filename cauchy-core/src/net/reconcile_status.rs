use secp256k1::PublicKey;
use crypto::signatures::ecdsa::generate_dummy_pubkey;

pub struct ReconciliationStatus {
    live: bool,
    target: PublicKey
}

impl ReconciliationStatus {
    pub fn new() -> ReconciliationStatus {
        let dummy_pk = generate_dummy_pubkey();
        ReconciliationStatus {
            live: false,
            target: dummy_pk
        }
    }

    pub fn is_live(&self) -> bool {
        self.live
    }

    pub fn stop(&mut self) {
        self.live = false;
    }

    pub fn set_target(&mut self, new_target: &PublicKey) {
        self.live = true;
        self.target = *new_target;
    }

    pub fn eq(&self, other: &PublicKey) -> bool {
        self.target == *other
    }
}