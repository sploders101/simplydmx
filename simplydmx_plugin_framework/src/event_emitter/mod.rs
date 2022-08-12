mod arc_portable;
mod event_receiver;
mod portable_message;

use std::{
	sync::Arc,
	collections::HashMap,
	any::TypeId,
};

use async_std::{
	task,
	channel::{
		self,
		Sender,
		Receiver,
	},
};

use simplydmx_plugin_macros::portable;

pub use portable_message::{
	PortableMessage,
	BidirectionalPortable,
	PortableMessageDeserializer,
	PortableMessageGenericDeserializer,
};

pub use event_receiver::EventReceiver;
pub use arc_portable::ArcPortable;
pub use event_receiver::Event;

type PortableEvent = PortableEventGeneric<Box<dyn PortableMessage>>;
type PortableJSONEvent = PortableEventGeneric<serde_json::Value>;
type PortableBincodeEvent = PortableEventGeneric<Vec<u8>>;
pub enum PortableEventGeneric<T> {
	Msg(Arc<T>),
	Shutdown,
}


/// # Semi-statically-typed event bus.
///
/// The `EventEmitter` allows distribution of data by two keys: event name and TypeId.
/// Currently, all data sent through the bus is of type `Any`.
///
/// ## Implementation
///
/// When data is sent on the bus, it is cast to `Any` through the use of a generic
/// function. That value is sent through the bus to all listeners subscribed to that
/// particular event, and is wrapped by an `ArcPortable` of the desired type, allowing
/// easy use of the value as its intended type without cloning or usage of the `Any`
/// value directly.
///
/// ## Type semantics
///
/// TL;DR: Use `#[simplydmx_plugin_macros::portable]` on all types that should traverse the bus.
///
/// Event channels are statically typed. This is done using `TypeId`s from the built-in
/// Any trait. All events traversing the bus must implement BidirectionalPortable, which
/// provides a common API for serialization and deserialization without type information.
///
/// BidirectionalPortablle is used as a marker to indicate that a type has implemented
/// both serde's `Serialize` and `Deserialize` traits, and can/has therefore been expanded
/// to create a type-erased serialization and deserialization API.
///
/// `simplydmx_plugin_macros` includes a `portable` macro that will specify all the necessary
/// derives for your convenience.
pub struct EventEmitter {
	listeners: HashMap<String, ListenerInfo>,

	shutdown_sender: Sender<()>,
}

impl EventEmitter {

	/// Creates a new EventEmitter.
	pub fn new() -> (EventEmitter, Receiver<()>) {
		let (sender, receiver) = channel::bounded(1);
		return (EventEmitter {
			listeners: HashMap::new(),
			shutdown_sender: sender,
		}, receiver);
	}

	/// Runs garbage collection for old receivers that are no longer active.
	/// This is kind of a scrappy implementation, but should work in the short-term
	/// to get an MVP running.
	fn gc(&mut self) {

		// Clear out listener Vecs
		let mut to_remove = Vec::new();
		for (event_id, listener_info) in self.listeners.iter_mut() {
			let mut i = 0;

			// PortableEvent Listeners
			let listeners = &mut listener_info.listeners;
			while i < listeners.len() {
				if listeners[i].receiver_count() == 0 {
					listeners.remove(i);
				} else {
					i += 1;
				}
			}

			// JSON Listeners
			let json_listeners = &mut listener_info.json_listeners;
			while i < json_listeners.len() {
				if json_listeners[i].receiver_count() == 0 {
					json_listeners.remove(i);
				} else {
					i += 1;
				}
			}

			// Bincode listeners
			let bincode_listeners = &mut listener_info.bincode_listeners;
			while i < bincode_listeners.len() {
				if bincode_listeners[i].receiver_count() == 0 {
					bincode_listeners.remove(i);
				} else {
					i += 1;
				}
			}

			// Mark listener entry for removal if it's empty and wasn't declared
			if !listener_info.persistent
				&& listeners.len() == 0
				&& json_listeners.len() == 0
				&& bincode_listeners.len() == 0
			{
				to_remove.push(String::clone(event_id));
			}
		}
		for event_id in to_remove {
			self.listeners.remove(&event_id);
		}

	}

	pub fn declare_event<T: BidirectionalPortable>(&mut self, event_name: String) -> Result<(), DeclareEventError> {
		// Check if event already exists
		if self.listeners.contains_key(&event_name) {
			let listener_info = self.listeners.get(&event_name).unwrap();
			if listener_info.type_id != TypeId::of::<T>() {
				return Err(DeclareEventError::AlreadyDeclared);
			}
		}

		// Create and insert ListenerInfo for declared event
		let new_listener_info = ListenerInfo::new::<T>();
		self.listeners.insert(event_name, new_listener_info);
		return Ok(());
	}

	/// Registers an event listener on the bus of the given type. Returns
	/// an instance of `EventReceiver<T>` which filters for the desired type
	/// and wraps resulting values in `ArcPortable<T>` to make usage of the data
	/// simpler.
	pub fn on<T: BidirectionalPortable>(&mut self, event_name: String) -> Result<EventReceiver<T>, RegisterListenerError> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			return Err(RegisterListenerError::NotDeclared);
		}

		let listener_info = self.listeners.get_mut(&event_name).unwrap();

		// Return error if types are incompatible. This is not required, but is done out of principle to
		// ensure event types are consistent, which will help later with automated self-documentation.
		if listener_info.type_id != TypeId::of::<T>() {
			return Err(RegisterListenerError::EventClaimedAsType);
		}

		let (sender, receiver) = channel::unbounded();

		listener_info.listeners.push(sender);
		return Ok(EventReceiver::new(event_name, receiver));
	}

	/// Registers a listener on the event bus that receives pre-encoded JSON events
	pub fn on_json(&mut self, event_name: String) -> Result<Receiver<PortableJSONEvent>, RegisterEncodedListenerError> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			return Err(RegisterEncodedListenerError::NotDeclared);
		}

		let listener_info = self.listeners.get_mut(&event_name).unwrap();

		let (sender, receiver) = channel::unbounded();
		listener_info.json_listeners.push(sender);
		return Ok(receiver);
	}

	/// Registers a listener on the bus that receives pre-encoded bincode events
	pub fn on_bincode(&mut self, event_name: String) -> Result<Receiver<PortableBincodeEvent>, RegisterEncodedListenerError> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			return Err(RegisterEncodedListenerError::NotDeclared);
		}

		let listener_info = self.listeners.get_mut(&event_name).unwrap();

		let (sender, receiver) = channel::unbounded();
		listener_info.bincode_listeners.push(sender);
		return Ok(receiver);
	}

	fn send_portablemessage(listener_info: &mut ListenerInfo, message: Box<dyn PortableMessage>) {
		let message_arc = Arc::new(message);
		for listener in listener_info.listeners.iter() {
			let listener_cloned = listener.clone();
			let message_arc_cloned = Arc::clone(&message_arc);
			task::spawn(async move {
				listener_cloned.send(PortableEvent::Msg(message_arc_cloned)).await.ok();
			});
		}
	}

	fn send_json(listener_info: &mut ListenerInfo, message: serde_json::Value) {
		let message_arc = Arc::new(message);
		for listener in listener_info.json_listeners.iter() {
			let listener_cloned = listener.clone();
			let message_arc_cloned = Arc::clone(&message_arc);
			task::spawn(async move {
				listener_cloned.send(PortableJSONEvent::Msg(message_arc_cloned)).await.ok();
			});
		}
	}

	fn send_bincode(listener_info: &mut ListenerInfo, message: Vec<u8>) {
		let message_arc = Arc::new(message);
		for listener in listener_info.bincode_listeners.iter() {
			let listener_cloned = listener.clone();
			let message_arc_cloned = Arc::clone(&message_arc);
			task::spawn(async move {
				listener_cloned.send(PortableBincodeEvent::Msg(message_arc_cloned)).await.ok();
			});
		}
	}

	/// Sends an event on the bus. `T` gets cast to `Any`, boxed, wrapped in `Arc`,
	/// and sent to all registered listeners.
	pub async fn emit<T: BidirectionalPortable>(&mut self, event_name: String, message: T) {
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {
			// Re-broadcast JSON
			if listeners.json_listeners.len() > 0 {
				if let Ok(translated) = message.serialize_json() {
					Self::send_json(listeners, translated);
				}
			}

			// Re-broadcast bincode
			if listeners.bincode_listeners.len() > 0 {
				if let Ok(translated) = message.serialize_bincode() {
					Self::send_bincode(listeners, translated);
				}
			}

			Self::send_portablemessage(listeners, Box::new(message));
		}

	}

	/// Emits a JSON value to the bus, deserializing for listeners of other formats if
	/// necessary/possible. It will always be repeated to JSON listeners, but will silently
	/// fail to repeat on listeners of other protocols if deserialization fails
	pub async fn emit_json(&mut self, event_name: String, message: serde_json::Value) {
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {

			let deserialized: Option<Box<dyn PortableMessage>> = if
				listeners.listeners.len() > 0
				|| listeners.bincode_listeners.len() > 0
			{ listeners.deserializer.deserialize_json(message.clone()).ok() }
			else { None };

			// Re-broadcast JSON
			if listeners.json_listeners.len() > 0 {
				Self::send_json(listeners, message.clone());
			}

			// Re-broadcast bincode
			if listeners.bincode_listeners.len() > 0 {
				if let Ok(translated) = deserialized.as_ref().unwrap().serialize_bincode() {
					Self::send_bincode(listeners, translated);
				}
			}

			// Deserialized
			if let Some(deserialized) = deserialized {
				Self::send_portablemessage(listeners, deserialized);
			}

		}
	}

	/// Emits a Bincode value to the bus, deserializing for listeners of other formats if
	/// necessary/possible. It will always be repeated to Bincode listeners, but will silently
	/// fail to repeat on listeners of other protocols if deserialization fails
	pub async fn emit_bincode(&mut self, event_name: String, message: Vec<u8>) {
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {

			let deserialized: Option<Box<dyn PortableMessage>> = if
				listeners.listeners.len() > 0
				|| listeners.json_listeners.len() > 0
			{ listeners.deserializer.deserialize_bincode(&message).ok() }
			else { None };

			// Re-broadcast JSON
			if listeners.json_listeners.len() > 0 {
				if let Ok(translated) = deserialized.as_ref().unwrap().serialize_json() {
					Self::send_json(listeners, translated);
				}
			}

			// Re-broadcast bincode
			if listeners.bincode_listeners.len() > 0 {
				Self::send_bincode(listeners, message);
			}

			// Deserialized
			if let Some(deserialized) = deserialized {
				Self::send_portablemessage(listeners, deserialized);
			}

		}
	}

	pub async fn send_shutdown(&mut self) {
		self.gc();

		self.shutdown_sender.send(()).await.ok();
		for listener_group in self.listeners.values() {
			for listener in listener_group.listeners.iter() {
				let listener_cloned = listener.clone();
				task::spawn(async move {
					listener_cloned.send(PortableEvent::Shutdown).await.ok();
				});
			}
		}
	}
}

/// This struct allows events to be staked as a specific type to ensure consistency within the API.
/// There is no technical limitation that warrants this, but instead, it is used to make sure that
/// self-documenting routines are accurate and simple. It also ensures that similar-but-different
/// types don't get confused or create race conditions when deserialized.
pub struct ListenerInfo {
	pub type_id: TypeId,
	pub persistent: bool,
	pub deserializer: Box<dyn PortableMessageGenericDeserializer>,
	pub listeners: Vec<Sender<PortableEvent>>,
	pub json_listeners: Vec<Sender<PortableJSONEvent>>,
	pub bincode_listeners: Vec<Sender<PortableBincodeEvent>>,
}

impl ListenerInfo {
	pub fn new<T: BidirectionalPortable>() -> Self {
		return ListenerInfo {
			persistent: true,
			type_id: TypeId::of::<T>(),
			deserializer: Box::new(PortableMessageDeserializer::<T>::new()),
			listeners: Vec::new(),
			json_listeners: Vec::new(),
			bincode_listeners: Vec::new(),
		};
	}
}

#[portable]
pub enum DeclareEventError {
	AlreadyDeclared,
}

#[portable]
pub enum RegisterListenerError {
	NotDeclared,
	EventClaimedAsType,
}

#[portable]
pub enum RegisterEncodedListenerError {
	NotDeclared,
}
