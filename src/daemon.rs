use db::rocksdb::RocksDb;
use db::*;
use secp256k1::SecretKey;

use bytes::Bytes;
use crypto::signatures::ecdsa;
use std::env;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{lines, write_all};
use tokio::net::TcpListener;
use tokio::prelude::*;

use consensus::status::*;
use net::messages::*;
use primitives::transaction::Transaction;
use utils::serialisation::*;

pub fn response_server(tx_db: Arc<RocksDb>, my_status: Arc<Status>, my_sk: Arc<SecretKey>) {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr)
        .map_err(|_| "failed to bind")
        .unwrap();

    println!("Listening on: {}", addr);

    let rs = listener
        .incoming()
        .map_err(|e| println!("error accepting socket; error = {:?}", e))
        .for_each(move |socket| {
            let (reader, writer) = socket.split();
            let lines = lines(BufReader::new(reader));
            let db = tx_db.clone();
            let my_status = my_status.clone();
            let my_sk = my_sk.clone();

            let responses = lines.map(move |line| {
                let incoming_msgs = match Message::parse(Bytes::from(line.as_bytes())) {
                    Ok(req) => req,
                    Err(_) => {
                        return Message::Error {
                            msg: "Unknown request".to_string(),
                        }
                    }
                };

                match incoming_msgs {
                    Message::StartHandshake { preimage } => Message::EndHandshake {
                        pubkey: my_status.get_public_key(),
                        sig: ecdsa::sign(&ecdsa::message_from_preimage(preimage), &my_sk),
                    },
                    Message::GetTransaction { tx_id } => match db.get(&tx_id) {
                        Ok(Some(tx_raw)) => Message::Transaction {
                            tx: Transaction::try_from(tx_raw).unwrap(),
                        },
                        Ok(None) => Message::Error {
                            msg: "Failed to find transaction".to_string(),
                        },
                        Err(_) => Message::Error {
                            msg: "Database read error".to_string(),
                        },
                    },
                    Message::GetStateSketch => Message::StateSketch {
                        sketch: my_status.get_state_sketch(),
                    },
                    Message::GetNonce => Message::Nonce {
                        nonce: my_status.get_nonce(),
                    },
                    _ => Message::End,
                }
            });

            let writes = responses.fold(writer, |writer, response| {
                let response = response.serialise();
                write_all(writer, response).map(|(w, _)| w)
            });

            let msg = writes.then(move |_| Ok(()));

            tokio::spawn(msg)
        });

    tokio::run(rs);
}
