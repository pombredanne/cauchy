use futures::sync::mpsc::{channel, Receiver, Sender};
use net::messages::Message;
use primitives::arena::Arena;
use secp256k1::PublicKey;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

pub struct ConnectionManager {
    connections: HashMap<SocketAddr, ConnectionStatus>,
    router_sender: Sender<(SocketAddr, Message)>,
    router_receiver: Receiver<(SocketAddr, Message)>,
}

impl ConnectionManager {
    pub fn init() -> ConnectionManager {
        let (router_sender, router_receiver) = channel::<(SocketAddr, Message)>(128);

        ConnectionManager {
            connections: HashMap::new(),
            router_sender,
            router_receiver,
        }
    }

    pub fn get_socket_by_pk(&self, pk: PublicKey) -> Option<SocketAddr> {
        let (socket, pk) = self
            .connections
            .iter()
            .find(|(socket, conn_status)| pk == *conn_status.current_pk.read().unwrap())?;
        Some(*socket)
    }

    pub fn get_router_sender(&self) -> Sender<(SocketAddr, Message)>{
        self.router_sender.clone()
    }

    pub fn add(
        &mut self,
        addr: &SocketAddr,
        pk: Arc<RwLock<PublicKey>>,
    ) -> Result<(u64, Receiver<Message>), String> {
        if self.connections.contains_key(addr) {
            return Err("Connection already managed".to_string());
        }
        let (msg_sender, msg_receiver) = channel::<Message>(1);

        // Randomize secret
        let secret: u64 = 32;

        // Initiate handshake
        let sender = msg_sender
            .clone()
            .send(Message::StartHandshake { secret })
            .then(|tx| match tx {
                Ok(_tx) => {
                    println!("Sink flushed");
                    Ok(())
                }
                Err(e) => {
                    println!("Sink failed! {:?}", e);
                    Err(())
                }
            });
        self.connections
            .insert(*addr, ConnectionStatus::new(secret, msg_sender.clone(), pk));
        tokio::spawn(sender);

        Ok((secret, msg_receiver))
    }
}

struct ConnectionStatus {
    pub secret: u64,
    pub msg_sender: Sender<Message>,
    pub current_pk: Arc<RwLock<PublicKey>>, // TODO: Misbehaviour history etc
}

impl ConnectionStatus {
    pub fn new(
        secret: u64,
        msg_sender: Sender<Message>,
        current_pk: Arc<RwLock<PublicKey>>,
    ) -> ConnectionStatus {
        ConnectionStatus {
            secret,
            msg_sender,
            current_pk,
        }
    }
}
