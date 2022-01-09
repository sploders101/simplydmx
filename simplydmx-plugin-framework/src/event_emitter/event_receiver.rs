use std::{
	marker::PhantomData,
	sync::Arc,
	any::{
		Any,
	},
};

use async_std::io;
use async_std::prelude::*;
use async_std::channel::{
	Receiver,
	RecvError,
};

use super::arc_any::ArcAny;

/// Wrapped receiver that filters an `Arc<Any>` event stream and returns `ArcAny<T>` for the
/// desired type. The `ArcAny<T>` allows the data to be easily cast to the correct type via deref
/// while maintaining ownership of the `Arc` to avoid deallocation.
pub struct EventReceiver<T: 'static> {
	event_name: String,
	receiver: Receiver<Arc<Box<dyn Any + Send + Sync>>>,
	_phantom: PhantomData<T>,
}

impl<T: 'static> EventReceiver<T> {

	/// Create a new Type filter that discards events that don't yield
	/// the desired type.
	///
	/// When the wrapped receiver returns an event with valid data, the data will be returned
	/// through an ArcAny struct. This struct maintains ownership of the arc while allowing a
	/// dereference to typed data. Type ID is checked on creation to prevent downcast issues.
	pub fn new(event_name: String, receiver: Receiver<Arc<Box<dyn Any + Send + Sync>>>) -> EventReceiver<T> {
		return EventReceiver::<T> {
			event_name,
			receiver,
			_phantom: PhantomData,
		};
	}

	/// Receives a single message of the desired type, wrapping it in an `ArcAny<T>` for ease
	/// of use.
	pub async fn receive(&self) -> Result<ArcAny<T>, RecvError> {
		loop {
			let msg = self.receiver.recv().await;
			match msg {
				Ok(data) => {
					if let Some(thing) = ArcAny::new(data) {
						return Ok(thing);
					} else {
						let mut stderr = io::stderr();
						stderr.write_all(format!("An event with id `{}` was dropped.", self.event_name).as_bytes());
					}
				},
				Err(error) => {
					return Err(error);
				}
			}
		}
	}
}
