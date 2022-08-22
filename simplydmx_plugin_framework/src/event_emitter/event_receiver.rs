use std::marker::PhantomData;

use async_std::channel::{
	Receiver,
};

use super::{
	PortableEvent,
	BidirectionalPortable,
	arc_portable::ArcPortable,
};

/// Wrapped receiver that filters an `Arc<Any>` event stream and returns `ArcPortable<T>` for the
/// desired type. The `ArcPortable<T>` allows the data to be easily cast to the correct type via deref
/// while maintaining ownership of the `Arc` to avoid deallocation.
pub struct EventReceiver<T: BidirectionalPortable> {
	event_name: String,
	receiver: Receiver<PortableEvent>,
	_phantom: PhantomData<T>,
}

pub enum Event<T: 'static> {
	Msg(ArcPortable<T>),
	Shutdown,
}

impl<T: BidirectionalPortable> EventReceiver<T> {

	/// Create a new Type filter that discards events that don't yield
	/// the desired type.
	///
	/// When the wrapped receiver returns an event with valid data, the data will be returned
	/// through an ArcPortable struct. This struct maintains ownership of the arc while allowing a
	/// dereference to typed data. Type ID is checked on creation to prevent downcast issues.
	pub fn new(event_name: String, receiver: Receiver<PortableEvent>) -> EventReceiver<T> {
		return EventReceiver::<T> {
			event_name,
			receiver,
			_phantom: PhantomData,
		};
	}

	pub fn get_name<'a>(&'a self) -> &'a String {
		return &self.event_name;
	}

	/// Receives a single message of the desired type, wrapping it in an `ArcPortable<T>` for ease
	/// of use.
	pub async fn receive(&self) -> Event<T> {
		loop {
			// Unwrapped because it *should* be impossible for the sender to be disconnected
			let msg = self.receiver.recv().await.unwrap();
			match msg {
				PortableEvent::Msg { data: msg } => {
					if let Some(msg) = ArcPortable::new(msg) {
						return Event::<T>::Msg(msg);
					}
					// else clause not needed; we will just wait for the next loop iteration
					// to find a value
				},
				PortableEvent::Shutdown => {
					return Event::<T>::Shutdown;
				}
			}
		}
	}
}
