use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use bus::BusReader;
use bytes::Bytes;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{Future, Sink};
use log::info;
use rand::Rng;
use secp256k1::{PublicKey, SecretKey, Signature};

use crate::{
    crypto::{
        hashes::Identifiable,
        signatures::ecdsa,
        sketches::{dummy_sketch::DummySketch, odd_sketch::OddSketch, SketchInsertable},
    },
    net::messages::*,
    utils::constants::{CONFIG, HASH_LEN},
};

use super::{transaction::Transaction, varint::VarInt, work_site::WorkSite};

macro_rules! ego_info {
    ($($arg:tt)*) => {
        if CONFIG.debugging.ego_verbose {
            info!(target: "ego_event", $($arg)*);
        }
    };
}

pub struct Ego {
    pubkey: PublicKey,
    seckey: SecretKey,

    oddsketch: OddSketch,
    minisketch: DummySketch,
    root: Bytes,
    nonce: u64,

    current_distance: u16,
}

pub trait WorkState {
    fn get_oddsketch(&self) -> OddSketch;
    fn get_root(&self) -> Bytes;
    fn get_nonce(&self) -> u64;
    fn update_oddsketch(&mut self, oddsketch: OddSketch);
    fn update_root(&mut self, root: Bytes);
    fn update_nonce(&mut self, nonce: u64);
}

impl WorkState for Ego {
    fn get_oddsketch(&self) -> OddSketch {
        self.oddsketch.clone()
    }

    fn get_root(&self) -> Bytes {
        self.root.clone()
    }

    fn get_nonce(&self) -> u64 {
        self.nonce
    }

    fn update_nonce(&mut self, new_nonce: u64) {
        self.nonce = new_nonce;
    }

    fn update_root(&mut self, root: Bytes) {
        self.root = root;
    }

    fn update_oddsketch(&mut self, oddsketch: OddSketch) {
        self.oddsketch = oddsketch;
    }
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
            current_distance: 512,
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

    pub fn get_work_site(&self) -> WorkSite {
        WorkSite::new(self.pubkey, self.root.clone(), self.nonce)
    }

    pub fn get_minisketch(&self) -> DummySketch {
        self.minisketch.clone()
    }

    pub fn update_current_distance(&mut self, new_distance: u16) {
        self.current_distance = new_distance;
    }

    pub fn get_current_distance(&self) -> u16 {
        self.current_distance
    }

    pub fn update_minisketch(&mut self, minisketch: DummySketch) {
        self.minisketch = minisketch;
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
    pub fn updater(
        ego: Arc<Mutex<Ego>>,
        distance_receive: std::sync::mpsc::Receiver<(u64, u16)>,
        mut mining_reset: BusReader<(OddSketch, Bytes)>,
    ) {
        let mut best_distance: u16 = 512;

        loop {
            if let Ok((nonce, distance)) = distance_receive.recv() {
                if mining_reset.try_recv().is_ok() {
                    let mut ego_locked = ego.lock().unwrap();
                    ego_locked.update_nonce(nonce);
                    ego_locked.update_current_distance(best_distance);
                    best_distance = distance;
                } else if distance < best_distance {
                    let mut ego_locked = ego.lock().unwrap();
                    ego_locked.update_nonce(nonce);
                    ego_locked.update_current_distance(best_distance);
                    best_distance = distance;
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

#[derive(PartialEq, Clone)]
pub enum WorkStatus {
    Waiting,
    Ready,
}

impl WorkStatus {
    pub fn to_str(&self) -> &'static str {
        match self {
            WorkStatus::Waiting => "waiting",
            WorkStatus::Ready => "ready",
        }
    }
}

impl Status {
    pub fn to_str(&self) -> &'static str {
        match self {
            Status::StatePush => "pushing",
            Status::StatePull => "pulling",
            Status::Gossiping => "gossiping",
        }
    }
}

#[derive(Clone)]
pub struct WorkStack {
    root: Bytes,
    nonce: u64,
    oddsketch: OddSketch,
    minisketch: DummySketch, // The minisketch to send to peer
}

impl WorkState for WorkStack {
    fn get_oddsketch(&self) -> OddSketch {
        self.oddsketch.clone()
    }
    fn get_root(&self) -> Bytes {
        self.root.clone()
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn update_oddsketch(&mut self, oddsketch: OddSketch) {
        self.oddsketch = oddsketch;
    }
    fn update_root(&mut self, root: Bytes) {
        self.root = root;
    }
    fn update_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
    }
}

impl WorkStack {
    fn get_minisketch(&self) -> DummySketch {
        self.minisketch.clone()
    }
}

#[derive(Default)]
pub struct Expectation {
    ids: Option<HashSet<Bytes>>,
    minisketch: Option<DummySketch>, // Post reconciliation our minisketch should match this
}

impl Expectation {
    pub fn get_ids(&self) -> Option<HashSet<Bytes>> {
        self.ids.clone()
    }

    pub fn get_minisketch(&self) -> Option<DummySketch> {
        self.minisketch.clone()
    }

    pub fn update_ids(&mut self, ids: HashSet<Bytes>) {
        self.ids = Some(ids)
    }

    pub fn update_minisketch(&mut self, minisketch: DummySketch) {
        self.minisketch = Some(minisketch)
    }

    pub fn is_expected_payload(&self, transactions: &HashSet<Transaction>) -> bool {
        Some(transactions.iter().map(|tx| tx.get_id()).collect()) == self.ids
    }

    pub fn clear_ids(&mut self) {
        self.ids = None
    }

    pub fn clear_minisketch(&mut self) {
        self.minisketch = None
    }
}

pub struct PeerEgo {
    pubkey: Option<PublicKey>,
    sink: Sender<Message>,
    secret: u64,
    status: Status,
    work_status: WorkStatus,

    // Reported
    reported_oddsketch: OddSketch,
    reported_root: Bytes,
    reported_nonce: u64,

    // Pre-acknowledgement perception
    pending: Option<WorkStack>,

    // How peer perceives own ego
    pub perception: Option<WorkStack>,

    // Expectation of state post-reconciliation
    pub expectation: Expectation,
}

impl WorkState for PeerEgo {
    fn get_oddsketch(&self) -> OddSketch {
        self.reported_oddsketch.clone()
    }

    fn get_nonce(&self) -> u64 {
        self.reported_nonce
    }

    fn get_root(&self) -> Bytes {
        self.reported_root.clone()
    }

    fn update_nonce(&mut self, new_nonce: u64) {
        self.reported_nonce = new_nonce;
    }

    fn update_root(&mut self, root: Bytes) {
        self.reported_root = root;
    }

    fn update_oddsketch(&mut self, oddsketch: OddSketch) {
        self.reported_oddsketch = oddsketch;
    }
}

impl PeerEgo {
    pub fn new() -> (PeerEgo, Receiver<Message>) {
        let (peer_sink, peer_stream) = channel::<Message>(1024); // TODO: Unbounded? Handle errors
        let mut rng = rand::thread_rng();
        (
            PeerEgo {
                pubkey: None,
                secret: rng.gen::<u64>(),
                reported_oddsketch: OddSketch::new(),
                reported_root: Bytes::from(&[0; HASH_LEN][..]),
                reported_nonce: 0,
                pending: None,
                work_status: WorkStatus::Ready,
                perception: None,
                status: Status::Gossiping,
                sink: peer_sink,
                expectation: Default::default(),
            },
            peer_stream,
        )
    }

    pub fn check_handshake(&mut self, sig: &Signature, pubkey: &PublicKey) {
        let secret_msg = ecdsa::message_from_preimage(Bytes::from(VarInt::new(self.secret)));
        if let Ok(true) = ecdsa::verify(&secret_msg, sig, pubkey) {
            self.pubkey = Some(*pubkey)
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

    pub fn get_work_status(&self) -> WorkStatus {
        self.work_status.clone()
    }

    pub fn get_pubkey(&self) -> Option<PublicKey> {
        self.pubkey
    }

    pub fn get_perceived_oddsketch(&self) -> Option<OddSketch> {
        match &self.perception {
            Some(perception) => Some(perception.get_oddsketch()),
            None => None,
        }
    }

    pub fn get_perceived_minisketch(&self) -> Option<DummySketch> {
        match &self.perception {
            Some(perception) => Some(perception.get_minisketch()),
            None => None,
        }
    }

    pub fn get_expected_minisketch(&self) -> Option<DummySketch> {
        self.expectation.get_minisketch()
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

    pub fn update_status(&mut self, status: Status) {
        ego_info!("{} -> {}", self.status.to_str(), status.to_str());
        self.status = status;
    }

    pub fn update_work_status(&mut self, work_status: WorkStatus) {
        ego_info!("{} -> {}", self.work_status.to_str(), work_status.to_str());
        self.work_status = work_status;
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

    // Update reported
    pub fn pull_work(&mut self, oddsketch: OddSketch, nonce: u64, root: Bytes) {
        self.reported_oddsketch = oddsketch;
        self.reported_nonce = nonce;
        self.reported_root = root;
    }

    // Update pending
    pub fn commit_work(&mut self, ego: &Ego) {
        // Send work
        self.pending = Some(WorkStack {
            root: ego.root.clone(),
            oddsketch: ego.oddsketch.clone(),
            nonce: ego.nonce,
            minisketch: ego.minisketch.clone(),
        });
    }

    pub fn push_work(&mut self) {
        // Confirm worked was received
        self.perception = self.pending.clone();
    }
}
