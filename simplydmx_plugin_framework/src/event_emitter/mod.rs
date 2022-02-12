mod arc_any;
mod event_receiver;

use std::{
	boxed::Box,
	sync::Arc,
	collections::HashMap,
	any::Any,
};

use async_std::{
	task,
	channel::{
		self,
		Sender,
	},
};

pub use event_receiver::EventReceiver;
pub use arc_any::ArcAny;


/// # Semi-statically-typed event bus.
///
/// The `EventEmitter` allows distribution of data by two keys: event name and TypeId.
/// Currently, all data sent through the bus is of type `Any`.
///
/// ## Implementation
///
/// When data is sent on the bus, it is cast to `Any` through the use of a generic
/// function. That value is sent through the bus to all listeners subscribed to that
/// particular event, and is wrapped by an `ArcAny` of the desired type, allowing
/// easy use of the value as its intended type without cloning or usage of the `Any`
/// value directly.
///
/// ## Type ambiguity
///
/// If the message is of any type other than what was requested, it is silently
/// ignored. This can be useful for implementing serialization and deserialization.
/// For example, a network-connected client may emit a JSON message on a certain event
/// channel. Whichever plugin recognizes the event name and message format can listen
/// for the serde `Value` type, then re-emit under its statically-typed equivalent.
pub struct EventEmitter {
	listeners: HashMap<String, Vec<Arc<Sender<Arc<Box<dyn Any + Send + Sync>>>>>>,
}

impl EventEmitter {

	/// Creates a new EventEmitter.
	pub fn new() -> EventEmitter {
		return EventEmitter {
			listeners: HashMap::new(),
		};
	}

	/// Runs garbage collection for old receivers that are no longer active.
	/// This is kind of a scrappy implementation, but should work in the short-term
	/// to get an MVP running.
	fn gc(&mut self) {

		let mut to_remove = Vec::new();

		// Clear out listener Vecs
		for (event_id, listeners) in self.listeners.iter_mut() {
			let mut i = 0;

			while i < listeners.len() {
				if listeners[i].receiver_count() == 0 {
					listeners.remove(i);
				} else {
					i += 1;
				}
			}

			if listeners.len() == 0 {
				to_remove.push(String::clone(event_id));
			}
		}

		// Remove events with no more listeners
		for event_id in to_remove {
			self.listeners.remove(&event_id);
		}

	}

	/// Registers an event listener on the bus of the given type. Returns
	/// an instance of `EventReceiver<T>` which filters for the desired type
	/// and wraps resulting values in `ArcAny<T>` to make usage of the data
	/// simpler.
	pub fn on<T>(&mut self, event_name: String) -> EventReceiver<T> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			self.listeners.insert(String::clone(&event_name), Vec::new());
		}

		let (sender, receiver) = channel::unbounded();

		self.listeners.get_mut(&event_name).unwrap().push(Arc::new(sender));
		return EventReceiver::new(event_name, receiver);
	}

	/// Sends an event on the bus. `T` gets cast to `Any`, boxed, wrapped in `Arc`,
	/// and sent to all registered listeners.
	pub async fn send<T>(&mut self, event_name: String, message: T)
	where
		T: Any + Send + Sync,
	{
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {
			let message_arc = Arc::<Box<dyn Any + Send + Sync>>::new(Box::new(message));
			for listener in listeners.iter() {
				let listener_cloned = Arc::clone(listener);
				let message_arc_cloned = Arc::clone(&message_arc);
				task::spawn(async move {
					listener_cloned.send(message_arc_cloned).await.ok();
				});
			}
		}
	}
}
