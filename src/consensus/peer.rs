use std::cell::{Cell, RefCell};
use bytes::{Bytes, BytesMut, BufMut};
use primitives::work_site::*;
use crypto::hashes::blake2b::*;
use utils::timing::*;

// TODO: Async
pub struct Peer {
	// TODO: Network params
	last_state_update: Cell<u64>,
	state_sketch: RefCell<BytesMut>,

	last_site_update: Cell<u64>,
	work_site: WorkSite,

	digested: Cell<bool>,
	work_digest: RefCell<BytesMut>,
}

impl Peer {
	pub fn update_work_digest(&self) {
		if !self.digested.get() {
			self.work_digest.borrow_mut().clear();
			self.work_digest.borrow_mut().put(self.work_site.to_bytes().blake2b());
			self.digested.set(true);
		}
	}

	pub fn update_state_sketch(&self, sketch: Bytes) {
		self.work_digest.borrow_mut().clear();
		self.work_digest.borrow_mut().put(sketch);
		self.last_state_update.set(get_current_time())
	}

	pub fn update_work_site(&self, nonce: u64) {
		self.work_site.set_nonce(nonce);
		self.digested.set(false);
		self.last_site_update.set(get_current_time());
	}

	pub fn get_state_sketch(&self) -> Bytes {
		self.state_sketch.borrow().clone().freeze()
	}

	pub fn get_work_digest(&self) -> Bytes {
		self.update_work_digest();
		self.work_digest.borrow().clone().freeze()
	}
}