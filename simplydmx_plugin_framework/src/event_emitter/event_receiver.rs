use std::{
	any::Any,
	marker::PhantomData,
};

use async_std::channel::{
	Receiver,
};

use serde::{
	Serialize,
	Deserialize,
};
use serde_json::Value;

use super::{
	AnyEvent,
	arc_any::ArcAny,
};

/// Wrapped receiver that filters an `Arc<Any>` event stream and returns `ArcAny<T>` for the
/// desired type. The `ArcAny<T>` allows the data to be easily cast to the correct type via deref
/// while maintaining ownership of the `Arc` to avoid deallocation.
pub struct EventReceiver<T: 'static> {
	event_name: String,
	receiver: Receiver<AnyEvent>,
	_phantom: PhantomData<T>,
}

pub enum Event<T: 'static> {
	Msg(ArcAny<T>),
	Shutdown,
}

impl<T: 'static + Any + Serialize + Deserialize<'static>> EventReceiver<T> {

	/// Create a new Type filter that discards events that don't yield
	/// the desired type.
	///
	/// When the wrapped receiver returns an event with valid data, the data will be returned
	/// through an ArcAny struct. This struct maintains ownership of the arc while allowing a
	/// dereference to typed data. Type ID is checked on creation to prevent downcast issues.
	pub fn new(event_name: String, receiver: Receiver<AnyEvent>) -> EventReceiver<T> {
		return EventReceiver::<T> {
			event_name,
			receiver,
			_phantom: PhantomData,
		};
	}

	pub fn get_name<'a>(&'a self) -> &'a String {
		return &self.event_name;
	}

	/// Receives a single message of the desired type, wrapping it in an `ArcAny<T>` for ease
	/// of use.
	pub async fn receive(&self) -> Event<T> {
		loop {
			// Unwrapped because it *should* be impossible for the sender to be disconnected
			let msg = self.receiver.recv().await.unwrap();
			match msg {
				AnyEvent::Msg(msg) => {
					if let Some(thing) = ArcAny::<T>::new(msg) {
						return Event::<T>::Msg(thing);
					}
				},
				AnyEvent::Shutdown => {
					return Event::<T>::Shutdown;
				}
			}
		}
	}
}
