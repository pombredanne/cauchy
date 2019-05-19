pub mod native;

use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use bytes::Bytes;
use futures::{sync::mpsc::Sender, Future};
use tokio::net::TcpStream;

use core::{
    daemon::{Origin, Priority},
    db::mongodb::MongoDB,
    primitives::{transaction::Transaction, tx_pool::TxPool},
};

#[macro_export]
macro_rules! rpc_info {
    ($($arg:tt)*) => {
        if config.debugging.rpc_verbose {
            info!(target: "rpc_event", $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! rpc_error {
    ($($arg:tt)*) => {
        if config.debugging.rpc_verbose {
            error!(target: "rpc_event", $($arg)*);
        }
    };
}

pub enum Request {
    AddPeer { addr: SocketAddr },               // 0 || Peer addr
    NewTransaction { tx: Transaction },         // 1 || Transaction
    FetchValue { actor_id: Bytes, key: Bytes }, // 2 || Actor ID || Key
}

pub enum Response {
    Success,
    Error,
    NotFound,
    Value(Bytes),
}

pub fn construct_rpc_stack(
    socket_sender: Sender<TcpStream>,
    mempool: Arc<Mutex<TxPool>>,
    db: MongoDB,
) -> Vec<Box<Future<Item = (), Error = ()> + Send + 'static>> {
    let mut stack: Vec<Box<Future<Item = (), Error = ()> + Send + 'static>> = Vec::new();

    #[cfg(feature = "native-rpc")]
    stack.push(native::interface::server(socket_sender, mempool, db));

    stack
}
