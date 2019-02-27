use bytes::Bytes;
use crypto::signatures::ecdsa;
use failure::Error;
use futures::sync::mpsc::{channel, Receiver, Sender};
use net::messages::Message;
use primitives::arena::Arena;
use primitives::varint::VarInt;
use rand::Rng;
use secp256k1::{PublicKey, Signature};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::net::TcpStream;
use tokio::prelude::*;
use utils::errors::{ConnectionAddError, ImpulseReceiveError, SocketNotFound};

pub struct ConnectionManager {
    connections: HashMap<SocketAddr, ConnectionStatus>,
    router_sender: Sender<(SocketAddr, Message)>,
    new_socket_send: Sender<TcpStream>,
}

impl ConnectionManager {
    pub fn init() -> (
        Arc<RwLock<ConnectionManager>>,
        Receiver<TcpStream>,
        impl Future<Item = (), Error = ()> + Send + 'static,
    ) {
        // Initialise the new peer stream
        let (new_socket_send, new_socket_recv) = channel::<TcpStream>(1);

        // Initialise connection manager
        let (router_sender, router_receiver) = channel::<(SocketAddr, Message)>(128);
        let cm = Arc::new(RwLock::new(ConnectionManager {
            connections: HashMap::new(),
            router_sender,
            new_socket_send,
        }));

        // Initialise router
        let cm_inner = cm.clone();
        let router = router_receiver.for_each(move |(socket_addr, message)| {
            // Fetch appropriate impulse sender from connection manager
            let impulse_sender = cm_inner
                .read()
                .unwrap()
                .get_impulse_sender(&socket_addr)
                .unwrap();

            // Route message to impulse sender
            let routed_send = impulse_sender.clone().send(message).then(|tx| match tx {
                Ok(_tx) => {
                    println!("Impulse sent");
                    Ok(())
                }
                Err(e) => {
                    println!("Impulse failed to send! {:?}", e);
                    Err(())
                }
            });
            tokio::spawn(routed_send)
        });
        (cm, new_socket_recv, router)
    }

    pub fn get_new_socket_send(&self) -> Sender<TcpStream> {
        self.new_socket_send.clone()
    }

    pub fn get_socket_by_pk(&self, pubkey: &PublicKey) -> Option<SocketAddr> {
        let (socket, _) = self
            .connections
            .iter()
            .find(|(_, conn_status)| *pubkey == *conn_status.pubkey.read().unwrap())?;
        Some(*socket)
    }

    pub fn get_impulse_sender(&self, socket_addr: &SocketAddr) -> Option<Sender<Message>> {
        let connection = self.connections.get(&socket_addr)?;
        Some(connection.impulse_sender.clone())
    }

    pub fn get_router_sender(&self) -> Sender<(SocketAddr, Message)> {
        self.router_sender.clone()
    }

    pub fn send_handshake(&mut self, addr: &SocketAddr) -> Result<(), SocketNotFound> {
        let status = match self.connections.get(&addr) {
            None => return Err(SocketNotFound), // TODO: From then ?
            Some(some) => some,
        };

        let secret = status.secret;
        self.send(addr, Message::StartHandshake { secret });

        Ok(())
    }

    pub fn send(&self, addr: &SocketAddr, message: Message) -> Result<(), SocketNotFound> {
        let impulse_sender = match self.get_impulse_sender(addr) {
            None => return Err(SocketNotFound), // TODO: From then ?
            Some(some) => some,
        };

        let handshake_impulse = impulse_sender.clone().send(message).then(|tx| match tx {
            Ok(_tx) => {
                println!("Handshake impulse sent");
                Ok(())
            }
            Err(e) => {
                println!("Handshake failed to send! {:?}", e);
                Err(())
            }
        });

        tokio::spawn(handshake_impulse);
        Ok(())
    }

    pub fn check_handshake(
        &mut self,
        arena: Arc<RwLock<Arena>>,
        addr: &SocketAddr,
        sig: &Signature,
        pubkey: &PublicKey,
    ) -> Result<bool, SocketNotFound> {
        let mut status = match self.connections.get_mut(&addr) {
            None => return Err(SocketNotFound), // TODO: From then ?
            Some(some) => some,
        };

        if status.check_sig(sig, pubkey) {
            let mut arena_write = arena.write().unwrap();
            (*arena_write).replace_key(&status.pubkey.read().unwrap(), &pubkey);
            drop(arena_write);
            status.replace_pubkey(pubkey);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn add(
        &mut self,
        addr: &SocketAddr,
    ) -> Result<
        (
            impl futures::stream::Stream<Item = Message, Error = Error>,
            Arc<RwLock<PublicKey>>,
        ),
        ConnectionAddError,
    > {
        if self.connections.contains_key(addr) {
            return Err(ConnectionAddError);
        }
        let (impulse_sender, impulse_receiver) = channel::<Message>(1);

        // Create random secret
        let mut rng = rand::thread_rng();
        let secret: u64 = rng.gen();

        // Generate dummy pubkey
        let dummy_pk = Arc::new(RwLock::new(ecdsa::generate_dummy_pubkey()));

        // Create connection status
        self.connections.insert(
            *addr,
            ConnectionStatus::new(secret, impulse_sender.clone(), dummy_pk.clone()),
        );

        let impulse_receiver = impulse_receiver.map_err(|_| ImpulseReceiveError.into());
        Ok((impulse_receiver, dummy_pk))
    }
}

struct ConnectionStatus {
    secret: u64,
    impulse_sender: Sender<Message>,
    pubkey: Arc<RwLock<PublicKey>>,
    // TODO: Misbehaviour history etc
    // TODO: Last handshake
}

impl ConnectionStatus {
    pub fn new(
        secret: u64,
        impulse_sender: Sender<Message>,
        pubkey: Arc<RwLock<PublicKey>>,
    ) -> ConnectionStatus {
        ConnectionStatus {
            secret,
            impulse_sender,
            pubkey,
        }
    }

    pub fn replace_pubkey(&mut self, pubkey: &PublicKey) {
        *self.pubkey.write().unwrap() = *pubkey;
    }

    pub fn get_secret(&self) -> u64 {
        self.secret
    }

    pub fn check_sig(&self, sig: &Signature, pubkey: &PublicKey) -> bool {
        let secret_msg = ecdsa::message_from_preimage(Bytes::from(VarInt::new(self.secret)));
        ecdsa::verify(&secret_msg, sig, pubkey).unwrap()
    }

    pub fn get_impulse_sender(&self) -> Sender<Message> {
        self.impulse_sender.clone()
    }
}
