use db::rocksdb::RocksDb;
use db::*;
use primitives::status::Status;
use secp256k1::{PublicKey, SecretKey};

use bytes::Bytes;
use crypto::signatures::ecdsa;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
use tokio::codec::Framed;
use tokio::net::TcpListener;
use tokio::prelude::*;

use net::messages::*;
use primitives::arena::Arena;
use primitives::transaction::Transaction;
use primitives::varint::VarInt;
use utils::serialisation::*;

pub fn response_server(
    tx_db: Arc<RocksDb>,
    self_status: Arc<Status>,
    local_pk: PublicKey,
    local_sk: SecretKey,
) {
    let mut arena = Arc::new(RwLock::new(Arena::new(&local_pk, self_status.clone())));

    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let self_secret_msg = 32;
    let self_secret = ecdsa::message_from_preimage(Bytes::from(VarInt::new(self_secret_msg)));

    let listener = TcpListener::bind(&addr)
        .map_err(|_| "failed to bind")
        .unwrap();

    let done = listener
        .incoming()
        .map_err(|e| println!("error accepting socket; error = {:?}", e))
        .for_each(move |socket| {
            println!("Found new socket!");

            let socket_pk = Arc::new(Mutex::new(local_pk)); //TODO: Proper dummy pubkey

            let framed_sock = Framed::new(socket, MessageCodec);
            let (sink, stream) = framed_sock.split();
            let tx_db_c = tx_db.clone();
            let self_status_c = self_status.clone();
            let arena_c = arena.clone();

            let queries = stream.filter(move |msg| match msg {
                Message::StartHandshake { secret } => true,
                Message::EndHandshake { pubkey, sig } => {
                    // Add peer to arena
                    let new_status = Arc::new(Status::null());
                    let mut arena_m = arena_c.write().unwrap();
                    if ecdsa::verify(&self_secret, sig, pubkey).unwrap() {
                        arena_m.add_peer(&pubkey, new_status);
                        let mut socket_pk_locked = socket_pk.lock().unwrap();
                        *socket_pk_locked = *pubkey;
                    }
                    false
                }
                Message::Nonce { nonce } => {
                    // Update nonce
                    let arena_r = arena_c.read().unwrap();
                    let socket_pk_locked = socket_pk.lock().unwrap();
                    let peer_status = arena_r.get_peer(&*socket_pk_locked);
                    peer_status.update_nonce(*nonce);
                    true // TODO: Under conditions
                }
                Message::OddSketch { sketch } => {
                    // Update statesketch
                    let arena_r = arena_c.read().unwrap();
                    let socket_pk_locked = socket_pk.lock().unwrap();
                    let peer_status = arena_r.get_peer(&*socket_pk_locked);
                    peer_status.update_odd_sketch(sketch.clone());
                    true // TODO: Under conditions
                }
                Message::IBLT { iblt } => {
                    let arena_r = arena_c.read().unwrap();
                    let socket_pk_locked = socket_pk.lock().unwrap();
                    let peer_status = arena_r.get_peer(&*socket_pk_locked);
                    peer_status.update_sketch(iblt.clone());
                    true
                }
                Message::GetTransactions { ids } => true,
                Message::Transactions { txs } => {
                    // TODO: Insert into database
                    false
                }
            });

            let responses = queries.map(move |msg| match msg {
                Message::StartHandshake { secret } => Message::EndHandshake {
                    pubkey: local_pk,
                    sig: ecdsa::sign(
                        &ecdsa::message_from_preimage(Bytes::from(VarInt::new(secret))),
                        &local_sk,
                    ),
                },
                Message::GetTransactions { ids } => {
                    let mut txs = Vec::with_capacity(ids.len());
                    for id in ids {
                        match tx_db_c.get(&id) {
                            Ok(Some(tx_raw)) => txs.push(Transaction::try_from(tx_raw).unwrap()),
                            _ => (),
                        }
                    }
                    Message::Transactions { txs }
                }
                Message::Nonce { nonce: _ } => Message::Nonce {
                    nonce: self_status_c.get_nonce(),
                },
                Message::OddSketch { sketch: _ } => Message::OddSketch {
                    sketch: self_status_c.get_odd_sketch(),
                },
                Message::IBLT { iblt: _ } => Message::IBLT {
                    iblt: self_status_c.get_sketch(),
                },
                _ => unreachable!(),
            });

            sink.send_all(responses)
                .map(|_| ())
                .map_err(|e| println!("error = {:?}", e))
        });
    tokio::run(done);
}
