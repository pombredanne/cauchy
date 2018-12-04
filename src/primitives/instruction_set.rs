use primitives::{instruction::{LocalEvaluation, GlobalEvaluation}, varint::VarInt};
use bytes::Bytes;
use db::Database;
use db::rocksdb::Rocksdb;
use primitives::transaction::Transaction;

struct Grab(bool);