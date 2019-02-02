use futures::sync::mpsc::{channel, Receiver, Sender};
use net::messages::Message;
use secp256k1::PublicKey;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::io::{Error, ErrorKind};
use tokio::prelude::*;

pub struct ConnectionManager {
    connections: HashMap<SocketAddr, ConnectionStatus>,
    router_sender: Sender<(SocketAddr, Message)>,
}

impl ConnectionManager {
    pub fn init() -> (Arc<RwLock<ConnectionManager>>, impl Future<Item = (), Error = ()> + Send + 'static) {
        // Init connection manager
        let (router_sender, router_receiver) = channel::<(SocketAddr, Message)>(128);
        let cm = Arc::new(RwLock::new(ConnectionManager {
            connections: HashMap::new(),
            router_sender,
        }));

        // Init router
        let cm_inner = cm.clone();
        let router = router_receiver
            .map(move |(socket_addr, message)| {
                let cm_read = &*cm_inner.read().unwrap();
                let msg_sender = cm_read.get_msg_sender(&socket_addr).unwrap();
                let routed_send = msg_sender.clone().send(message).then(|tx| match tx {
                    Ok(_tx) => {
                        println!("Sink flushed");
                        Ok(())
                    }
                    Err(e) => {
                        println!("Sink failed! {:?}", e);
                        Err(())
                    }
                });
                tokio::spawn(routed_send)
            })
            .into_future()
            .map(|_| ())
            .map_err(|_| ());
        (cm, router)
    }

    pub fn get_socket_by_pk(&self, pk: PublicKey) -> Option<SocketAddr> {
        let (socket, pk) = self
            .connections
            .iter()
            .find(|(socket, conn_status)| pk == *conn_status.current_pk.read().unwrap())?;
        Some(*socket)
    }

    pub fn get_msg_sender(&self, socket_addr: &SocketAddr) -> Option<Sender<Message>> {
        let connection = self.connections.get(&socket_addr)?;
        Some(connection.msg_sender.clone())
    }

    pub fn get_router_sender(&self) -> Sender<(SocketAddr, Message)> {
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
        let handshake_send = msg_sender
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
        tokio::spawn(handshake_send);

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
