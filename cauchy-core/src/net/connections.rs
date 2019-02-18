use futures::sync::mpsc::{channel, Sender};
use net::messages::Message;
use secp256k1::PublicKey;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use failure::Error;
use utils::errors::{ConnectionAddError, ImpulseReceiveError};

pub struct ConnectionManager {
    connections: HashMap<SocketAddr, ConnectionStatus>,
    router_sender: Sender<(SocketAddr, Message)>,
}

impl ConnectionManager {
    pub fn init() -> (
        Arc<RwLock<ConnectionManager>>,
        impl Future<Item = (), Error = ()> + Send + 'static,
    ) {
        // Initialise connection manager
        let (router_sender, router_receiver) = channel::<(SocketAddr, Message)>(128);
        let cm = Arc::new(RwLock::new(ConnectionManager {
            connections: HashMap::new(),
            router_sender,
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
        (cm, router)
    }

    pub fn get_socket_by_pk(&self, pk: PublicKey) -> Option<SocketAddr> {
        let (socket, _) = self
            .connections
            .iter()
            .find(|(_, conn_status)| pk == *conn_status.current_pk.read().unwrap())?;
        Some(*socket)
    }

    pub fn get_impulse_sender(&self, socket_addr: &SocketAddr) -> Option<Sender<Message>> {
        let connection = self.connections.get(&socket_addr)?;
        Some(connection.impulse_sender.clone())
    }

    pub fn get_router_sender(&self) -> Sender<(SocketAddr, Message)> {
        self.router_sender.clone()
    }

    pub fn add(
        &mut self,
        addr: &SocketAddr,
        pk: Arc<RwLock<PublicKey>>,
    ) -> Result<
        (
            u64,
            impl futures::stream::Stream<Item = Message, Error = Error>,
        ),
        ConnectionAddError,
    > {
        if self.connections.contains_key(addr) {
            return Err(ConnectionAddError);
        }
        let (impulse_sender, impulse_receiver) = channel::<Message>(1);

        // TODO: Randomize secret
        let secret: u64 = 32;

        // Initiate handshake
        let handshake_impulse = impulse_sender
            .clone()
            .send(Message::StartHandshake { secret })
            .then(|tx| match tx {
                Ok(_tx) => {
                    println!("Handshake impulse sent");
                    Ok(())
                }
                Err(e) => {
                    println!("Handshake failed to send! {:?}", e);
                    Err(())
                }
            });
        self.connections.insert(
            *addr,
            ConnectionStatus::new(secret, impulse_sender.clone(), pk),
        );
        tokio::spawn(handshake_impulse);

        let impulse_receiver = impulse_receiver.map_err(|_| ImpulseReceiveError.into());
        Ok((secret, impulse_receiver))
    }
}

struct ConnectionStatus {
    pub secret: u64,
    pub impulse_sender: Sender<Message>,
    pub current_pk: Arc<RwLock<PublicKey>>, // TODO: Misbehaviour history etc
}

impl ConnectionStatus {
    pub fn new(
        secret: u64,
        impulse_sender: Sender<Message>,
        current_pk: Arc<RwLock<PublicKey>>,
    ) -> ConnectionStatus {
        ConnectionStatus {
            secret,
            impulse_sender,
            current_pk,
        }
    }
}
