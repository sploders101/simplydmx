use std::borrow::Borrow;

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone, Error)]
pub enum IdAllocError {
	#[error("Out of available IDs")]
	Exhausted,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct IdAllocator {
	counter: u32,
	ids: FxHashSet<u32>,
}
impl IdAllocator {
	pub fn new() -> IdAllocator {
		return IdAllocator::default();
	}
	pub fn alloc(&mut self) -> Result<Id, IdAllocError> {
		if self.ids.len() == u32::MAX as usize {
			return Err(IdAllocError::Exhausted);
		}
		loop {
			if self.ids.contains(&self.counter) {
				continue;
			}

			let id = Id(self.counter);
			self.counter = self.counter.wrapping_add(1);
			return Ok(id);
		}
	}
	pub fn dealloc(&mut self, id: Id) {
		self.ids.remove(&id.0);
	}
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[must_use = "Ids need to be de-allocated so they can be re-used"]
pub struct Id(u32);
pub type IdRaw = u32;
impl Borrow<u32> for Id {
	fn borrow(&self) -> &u32 {
		return &self.0;
	}
}
impl Into<u32> for &Id {
	fn into(self) -> u32 {
		return self.0;
	}
}
