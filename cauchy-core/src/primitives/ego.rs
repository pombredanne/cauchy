use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use bus::Bus;
use bytes::Bytes;
use crossbeam::channel;
use crossbeam::channel::select;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{Future, Sink};
use secp256k1::{PublicKey, SecretKey, Signature};

use crypto::hashes::Identifiable;
use crypto::signatures::ecdsa;
use crypto::sketches::dummy_sketch::DummySketch;
use crypto::sketches::odd_sketch::OddSketch;
use crypto::sketches::SketchInsertable;
use net::messages::*;
use primitives::transaction::Transaction;
use primitives::varint::VarInt;
use primitives::work_site::WorkSite;
use utils::constants::HASH_LEN;

pub struct Ego {
    pubkey: PublicKey,
    seckey: SecretKey,

    oddsketch: OddSketch,
    minisketch: DummySketch,
    root: Bytes,
    nonce: u64,
}

impl Ego {
    pub fn new(pubkey: PublicKey, seckey: SecretKey) -> Ego {
        Ego {
            pubkey,
            seckey,
            oddsketch: OddSketch::new(),
            minisketch: DummySketch::new(),
            root: Bytes::from(&[0; HASH_LEN][..]),
            nonce: 0,
        }
    }

    pub fn generate_end_handshake(&self, secret: u64) -> Message {
        Message::EndHandshake {
            pubkey: self.pubkey,
            sig: ecdsa::sign(
                &ecdsa::message_from_preimage(Bytes::from(VarInt::new(secret))),
                &self.seckey,
            ),
        }
    }

    pub fn get_pubkey(&self) -> PublicKey {
        self.pubkey
    }

    pub fn get_oddsketch(&self) -> OddSketch {
        self.oddsketch.clone()
    }

    pub fn get_root(&self) -> Bytes {
        self.root.clone()
    }

    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }

    pub fn get_minisketch(&self) -> DummySketch {
        self.minisketch.clone()
    }

    pub fn update_nonce(&mut self, new_nonce: u64) {
        self.nonce = new_nonce;
    }

    pub fn increment(&mut self, new_tx: &Transaction, new_root: Bytes) {
        self.oddsketch.insert(new_tx);
        self.minisketch.insert(new_tx);
        self.root = new_root;
    }

    pub fn pull(&mut self, oddsketch: OddSketch, minisketch: DummySketch, root: Bytes) {
        self.oddsketch = oddsketch;
        self.minisketch = minisketch;
        self.root = root;
    }

    // Mining updates
    pub fn mining_updater(
        ego: Arc<Mutex<Ego>>,
        mut oddsketch_bus: Bus<(OddSketch, Bytes)>,
        tx_receive: channel::Receiver<Transaction>,
        distance_receive: channel::Receiver<(u64, u16)>,
    ) {
        let mut best_distance: u16 = 512;
        loop {
            select! {
                recv(tx_receive) -> tx => {
                    let mut ego_locked = ego.lock().unwrap();
                    let root = Bytes::from(&[0; 32][..]); // TODO: Actually get root
                    ego_locked.increment(&tx.unwrap(), root.clone());
                    oddsketch_bus.broadcast((ego_locked.get_oddsketch(), root));
                    best_distance = 512;
                },
                recv(distance_receive) -> pair => {
                    let (nonce, distance) = pair.unwrap();
                    if distance < best_distance {
                        ego.lock().unwrap().update_nonce(nonce);
                        best_distance = distance;
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum Status {
    StatePush,
    StatePull,
    Gossiping,
}

pub struct PeerEgo {
    pubkey: Option<PublicKey>,
    sink: Sender<Message>,
    secret: u64,

    // Reported
    reported_oddsketch: OddSketch,
    reported_root: Bytes,
    reported_nonce: u64,

    // Perceived
    perceived_root: Bytes,
    perceived_nonce: u64,
    perceived_oddsketch: OddSketch,

    // Anticipated
    anticipated_minisketch: DummySketch, // The minisketch to send to peer

    // Reconciliation
    status: Status,
    expected_ids: Option<HashSet<Bytes>>,
    expected_minisketch: Option<DummySketch>, // Post reconciliation our minisketch should match this
}

impl PeerEgo {
    pub fn new() -> (PeerEgo, Receiver<Message>) {
        let (peer_sink, peer_stream) = channel::<Message>(1024); // TODO: Unbounded? Handle errors
        (
            PeerEgo {
                pubkey: None,
                reported_oddsketch: OddSketch::new(),
                reported_root: Bytes::from(&[0; HASH_LEN][..]),
                reported_nonce: 0,
                perceived_root: Bytes::from(&[0; HASH_LEN][..]),
                perceived_nonce: 0,
                perceived_oddsketch: OddSketch::new(),
                anticipated_minisketch: DummySketch::new(),
                status: Status::Gossiping,
                sink: peer_sink,
                secret: 1337, // TODO: Randomize
                expected_ids: None,
                expected_minisketch: None,
            },
            peer_stream,
        )
    }

    pub fn check_handshake(&mut self, sig: &Signature, pubkey: &PublicKey) {
        let secret_msg = ecdsa::message_from_preimage(Bytes::from(VarInt::new(self.secret)));
        match ecdsa::verify(&secret_msg, sig, pubkey) {
            Ok(true) => self.pubkey = Some(*pubkey),
            _ => (), // TODO: Ban here?
        }
    }

    pub fn get_sink(&self) -> Sender<Message> {
        self.sink.clone()
    }

    pub fn get_secret(&self) -> u64 {
        self.secret
    }

    pub fn get_status(&self) -> Status {
        self.status.clone()
    }

    pub fn get_pubkey(&self) -> Option<PublicKey> {
        self.pubkey
    }

    pub fn get_oddsketch(&self) -> OddSketch {
        self.reported_oddsketch.clone()
    }

    pub fn get_nonce(&self) -> u64 {
        self.reported_nonce
    }

    pub fn get_root(&self) -> Bytes {
        self.reported_root.clone()
    }

    pub fn get_perceived_oddsketch(&self) -> OddSketch {
        self.perceived_oddsketch.clone()
    }

    pub fn get_anticipated_minisketch(&self) -> DummySketch {
        self.anticipated_minisketch.clone() // TODO: Catch? This panics if reconcile before work is sent
    }

    pub fn get_expected_minisketch(&self) -> DummySketch {
        self.expected_minisketch.clone().unwrap() // TODO: Catch? This panics if reconcile before work is sent
    }

    pub fn is_expected_payload(&self, transactions: &HashSet<Transaction>) -> bool {
        Some(transactions.iter().map(|tx| tx.get_id()).collect()) == self.expected_ids
    }

    pub fn get_work_site(&self) -> Option<WorkSite> {
        match self.pubkey {
            Some(pubkey) => Some(WorkSite::new(
                pubkey,
                self.reported_root.clone(),
                self.reported_nonce,
            )),
            None => None,
        }
    }

    // Update expected IDs
    pub fn update_ids(&mut self, ids: HashSet<Bytes>) {
        self.expected_ids = Some(ids)
    }

    pub fn update_status(&mut self, status: Status) {
        self.status = status;
    }

    // Update expected minisketch
    pub fn update_expected_minisketch(&mut self, minisketch: DummySketch) {
        self.expected_minisketch = Some(minisketch)
    }

    // On received
    // Receive nonce
    pub fn pull_nonce(&mut self, nonce: u64) {
        self.reported_nonce = nonce
    }

    pub fn send_msg(&self, message: Message) {
        tokio::spawn(
            self.sink
                .clone()
                .send(message)
                .and_then(|_| futures::future::ok(()))
                .map_err(|_| panic!()),
        );
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status
    }

    // Update reported
    pub fn pull_work(&mut self, oddsketch: OddSketch, nonce: u64, root: Bytes) {
        self.reported_oddsketch = oddsketch;
        self.reported_nonce = nonce;
        self.reported_root = root;
    }

    // Update perception
    pub fn push_work(&mut self, ego: &Ego) {
        // Send work
        self.perceived_root = ego.root.clone();
        self.perceived_oddsketch = ego.oddsketch.clone();
        self.perceived_nonce = ego.nonce;
        self.anticipated_minisketch = ego.minisketch.clone();
    }
}
