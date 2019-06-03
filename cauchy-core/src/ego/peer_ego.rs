use std::collections::HashSet;

use bytes::Bytes;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{Future, Sink};
use log::info;
use rand::Rng;
use secp256k1::{PublicKey, SecretKey, Signature};
use std::ops::Deref;

use crate::{
    crypto::{
        hashes::Identifiable,
        signatures::ecdsa,
        sketches::{dummy_sketch::DummySketch, odd_sketch::OddSketch, SketchInsertable},
    },
    ego::ego::Ego,
    net::messages::*,
    primitives::{
        status::{Expectation, PeerStatus},
        transaction::Transaction,
        varint::VarInt,
        work::{WorkSite, WorkStack, WorkState},
    },
    utils::constants::{CONFIG, HASH_LEN},
};

pub struct Perception {
    work_stack: WorkStack,
    minisketch: DummySketch,
}

impl Deref for Perception {
    type Target = WorkStack;
    fn deref(&self) -> &WorkStack {
        &self.work_stack
    }
}

impl Perception {
    pub fn new() {}

    pub fn get_minisketch(&self) -> DummySketch {
        self.minisketch.clone()
    }
}

pub struct PeerEgo {
    pubkey: Option<PublicKey>,
    sink: Sender<Message>,
    secret: u64,
    status: PeerStatus,

    // How peer perceives own ego
    pub perception: Option<Perception>,
}

impl PeerEgo {
    pub fn new() -> (PeerEgo, Receiver<Message>) {
        let (peer_sink, peer_stream) = channel::<Message>(1024); // TODO: Unbounded? Handle errors
        let mut rng = rand::thread_rng();
        (
            PeerEgo {
                pubkey: None,
                secret: rng.gen::<u64>(),
                perception: None,
                status: Default::default(),
                sink: peer_sink,
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

    pub fn get_status(&self) -> PeerStatus {
        self.status.clone()
    }

    pub fn get_status_mut(&mut self) -> &mut PeerStatus {
        &mut self.status
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

    pub fn update_status(&mut self, status: PeerStatus) {
        info!("{} -> {}", self.status.to_str(), status.to_str());
        self.status = status;
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
    pub fn pull_work(&mut self, work_stack: WorkStack) {
        self.status = PeerStatus::Fighting(work_stack);
    }

    // Update pending
    pub fn push_work(&mut self, work_stack: WorkStack, minisketch: DummySketch) {
        // Send work
        self.perception = Some(Perception {
            work_stack: work_stack.clone(),
            minisketch,
        });
        self.send_msg(Message::Work(work_stack))
    }
}
