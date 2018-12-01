use bytes::Bytes;
use db::Database;
use futures::prelude::*;

// TODO: Rewrite with futures/tokio where pulls can be done asyncronously?
pub trait LocalEvaluation {
    // Use what's ahead on the stack
    fn evaluate(&self, input: &Bytes) -> Result<Bytes, String>;
}

pub trait GlobalEvaluation<DB> {
    // Has access to a database
    fn evaluate(&self, db: &DB, input: &Bytes) -> Result<Bytes, String>;
}