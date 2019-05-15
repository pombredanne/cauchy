// #[cfg(feature = "native-rpc")]
pub mod native;

use std::collections::HashSet;

use futures::{sync::mpsc::Sender, Future};
use tokio::net::TcpStream;

use core::{
    daemon::{Origin, Priority},
    db::mongodb::MongoDB,
    primitives::transaction::Transaction,
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

pub fn construct_rpc_stack(
    socket_sender: Sender<TcpStream>,
    to_stage: Sender<(Origin, HashSet<Transaction>, Priority)>,
    tx_db: MongoDB,
) -> Vec<impl Future<Item = (), Error = ()> + Send + 'static> {
    let mut stack: Vec<Box<Future<Item = (), Error = ()> + Send + 'static>> = Vec::new();

    #[cfg(feature = "native-rpc")]
    stack.push(native::server);

    stack
}
