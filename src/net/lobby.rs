use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

struct Lobby {
    queue: Arc<Mutex<VecDeque<Bytes>>>,
}

impl Lobby {
    fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn pop(&self) -> Result<Option<Bytes>, String> {
        let maybe_queue = self.queue.lock();
        if let Ok(mut queue) = maybe_queue {
            Ok(queue.pop_front())
        } else {
            Err("Lobby tried to lock a poisoned mutex".to_string())
        }
    }

    fn push(&self, work: Bytes) -> Result<(), String> {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push_back(work);
            Ok(())
        } else {
            Err("Lobby tried to lock a poisoned mutex".to_string())
        }
    }
}
