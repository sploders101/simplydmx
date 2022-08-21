mod arc_portable;
mod event_receiver;
mod portable_message;

use std::{
	sync::Arc,
	collections::HashMap,
	any::TypeId,
};

use async_std::{
	channel::{
		self,
		Sender,
		Receiver,
	},
};

use uuid::Uuid;

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
pub type PortableJSONEvent = PortableEventGeneric<serde_json::Value>;
pub type PortableBincodeEvent = PortableEventGeneric<Vec<u8>>;
pub enum PortableEventGeneric<T: Sync + Send> {
	Msg(Arc<T>),
	Shutdown,
}

impl<T: Sync + Send> Clone for PortableEventGeneric<T> {
	fn clone(&self) -> Self {
		return match self {
			&PortableEventGeneric::Msg(ref message_arc) => { PortableEventGeneric::Msg(Arc::clone(message_arc)) },
			&PortableEventGeneric::Shutdown => { PortableEventGeneric::Shutdown },
		};
	}
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
				if listeners[i].1.receiver_count() == 0 {
					listeners.remove(i);
				} else {
					i += 1;
				}
			}

			// JSON Listeners
			let json_listeners = &mut listener_info.json_listeners;
			while i < json_listeners.len() {
				if json_listeners[i].1.receiver_count() == 0 {
					json_listeners.remove(i);
				} else {
					i += 1;
				}
			}

			// Bincode listeners
			let bincode_listeners = &mut listener_info.bincode_listeners;
			while i < bincode_listeners.len() {
				if bincode_listeners[i].1.receiver_count() == 0 {
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

	/// Declares an event on the bus so it can be translated between data formats and included in self-documentation.
	///
	/// Events not declared here will not be handled by Rust, including translation between protocols. Pre-serialized
	/// data (JSON and bincode, for example) will be repeated verbatim on the bus for any listeners of the same protocol.
	///
	/// The type parameter is used to construct a generic deserializer used for translation.
	pub fn declare_event<T: BidirectionalPortable>(&mut self, event_name: String) -> Result<(), DeclareEventError> {
		// Check if event already exists
		if self.listeners.contains_key(&event_name) {
			let listener_info = self.listeners.get_mut(&event_name).unwrap();
			if let Some(ref evt_info) = listener_info.evt_info {
				if evt_info.type_id != TypeId::of::<T>() {
					return Err(DeclareEventError::AlreadyDeclared);
				} else {
					return Ok(());
				}
			} else {
				listener_info.declare::<T>();
				return Ok(());
			}
		} else {
			// Create and insert ListenerInfo for declared event
			let new_listener_info = ListenerInfo::new_declared::<T>();
			self.listeners.insert(event_name, new_listener_info);
			return Ok(());
		}
	}

	/// Registers an event listener on the bus of the given type. Returns
	/// an instance of `EventReceiver<T>` which filters for the desired type
	/// and wraps resulting values in `ArcPortable<T>` to make usage of the data
	/// simpler.
	pub fn on<T: BidirectionalPortable>(&mut self, event_name: String, filter: FilterCriteria) -> Result<EventReceiver<T>, RegisterListenerError> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			self.listeners.insert(String::clone(&event_name), ListenerInfo::new());
		}

		let listener_info = self.listeners.get_mut(&event_name).unwrap();

		// Return error if types are incompatible. This is not required, but is done out of principle to
		// ensure event types are consistent, which will help later with automated self-documentation.
		if let Some(ref evt_info) = listener_info.evt_info {
			if evt_info.type_id != TypeId::of::<T>() {
				return Err(RegisterListenerError::EventClaimedAsType);
			}
		}

		let (sender, receiver) = channel::unbounded();

		listener_info.listeners.push((filter, sender));
		return Ok(EventReceiver::new(event_name, receiver));
	}

	/// Registers a listener on the event bus that receives pre-encoded JSON events
	pub fn on_json(&mut self, event_name: String, filter: FilterCriteria) -> Result<Receiver<PortableJSONEvent>, RegisterEncodedListenerError> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			self.listeners.insert(String::clone(&event_name), ListenerInfo::new());
		}

		let listener_info = self.listeners.get_mut(&event_name).unwrap();

		let (sender, receiver) = channel::unbounded();
		listener_info.json_listeners.push((filter, sender));
		return Ok(receiver);
	}

	/// Registers a listener on the bus that receives pre-encoded bincode events
	pub fn on_bincode(&mut self, event_name: String, filter: FilterCriteria) -> Result<Receiver<PortableBincodeEvent>, RegisterEncodedListenerError> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			self.listeners.insert(String::clone(&event_name), ListenerInfo::new());
		}

		let listener_info = self.listeners.get_mut(&event_name).unwrap();

		let (sender, receiver) = channel::unbounded();
		listener_info.bincode_listeners.push((filter, sender));
		return Ok(receiver);
	}

	/// Sends an event on the bus. `T` gets cast to `Any`, boxed, wrapped in `Arc`,
	/// and sent to all registered listeners.
	pub async fn emit<T: BidirectionalPortable>(&mut self, event_name: String, filter: FilterCriteria, message: T) {
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {
			// Re-broadcast JSON
			if relevant_listener(&filter, &listeners.json_listeners) {
				if let Ok(translated) = message.serialize_json() {
					send_filtered(&filter, PortableJSONEvent::Msg(Arc::new(translated)), &listeners.json_listeners);
				}
			}

			// Re-broadcast bincode
			if relevant_listener(&filter, &listeners.bincode_listeners) {
				if let Ok(translated) = message.serialize_bincode() {
					send_filtered(&filter, PortableBincodeEvent::Msg(Arc::new(translated)), &listeners.bincode_listeners);
				}
			}

			send_filtered(&filter, PortableEvent::Msg(Arc::new(Box::new(message))), &listeners.listeners);
		}

	}

	/// Emits a JSON value to the bus, deserializing for listeners of other formats if
	/// necessary/possible. It will always be repeated to JSON listeners, but will silently
	/// fail to repeat on listeners of other protocols if deserialization fails
	pub async fn emit_json(&mut self, event_name: String, filter: FilterCriteria, message: serde_json::Value) {
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {

			let deserialized: Option<Box<dyn PortableMessage>> = if
				relevant_listener(&filter, &listeners.listeners)
				|| relevant_listener(&filter, &listeners.bincode_listeners)
			{ if let Some(ref evt_info) = listeners.evt_info { evt_info.deserializer.deserialize_json(message.clone()).ok() } else { None } }
			else { None };

			// Re-broadcast JSON
			if relevant_listener(&filter, &listeners.json_listeners) {
				send_filtered(&filter, PortableJSONEvent::Msg(Arc::new(message)), &listeners.json_listeners);
			}

			// Re-broadcast bincode
			if relevant_listener(&filter, &listeners.bincode_listeners) {
				if let Ok(translated) = deserialized.as_ref().unwrap().serialize_bincode() {
					send_filtered(&filter, PortableBincodeEvent::Msg(Arc::new(translated)), &listeners.bincode_listeners);
				}
			}

			// Deserialized
			if let Some(deserialized) = deserialized {
				send_filtered(&filter, PortableEvent::Msg(Arc::new(deserialized)), &listeners.listeners);
			}

		}
	}

	/// Emits a Bincode value to the bus, deserializing for listeners of other formats if
	/// necessary/possible. It will always be repeated to Bincode listeners, but will silently
	/// fail to repeat on listeners of other protocols if deserialization fails
	pub async fn emit_bincode(&mut self, event_name: String, filter: FilterCriteria, message: Vec<u8>) {
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {

			let deserialized: Option<Box<dyn PortableMessage>> = if
				relevant_listener(&filter, &listeners.listeners)
				|| relevant_listener(&filter, &listeners.json_listeners)
			{ if let Some(ref evt_info) = listeners.evt_info { evt_info.deserializer.deserialize_bincode(&message).ok() } else { None }}
			else { None };

			// Re-broadcast JSON
			if relevant_listener(&filter, &listeners.json_listeners) {
				if let Ok(translated) = deserialized.as_ref().unwrap().serialize_json() {
					send_filtered(&filter, PortableJSONEvent::Msg(Arc::new(translated)), &listeners.json_listeners);
				}
			}

			// Re-broadcast bincode
			if relevant_listener(&filter, &listeners.json_listeners) {
				send_filtered(&filter, PortableBincodeEvent::Msg(Arc::new(message)), &listeners.bincode_listeners);
			}

			// Deserialized
			if let Some(deserialized) = deserialized {
				send_filtered(&filter, PortableEvent::Msg(Arc::new(deserialized)), &listeners.listeners);
			}

		}
	}

	pub async fn send_shutdown(&mut self) {
		self.gc();

		self.shutdown_sender.send(()).await.ok();
		for listener_group in self.listeners.values() {
			send_shutdown(&listener_group.listeners).await;
			send_shutdown(&listener_group.json_listeners).await;
			send_shutdown(&listener_group.bincode_listeners).await;
		}
	}
}

/// This struct allows events to be staked as a specific type to ensure consistency within the API.
/// There is no technical limitation that warrants this, but instead, it is used to make sure that
/// self-documenting routines are accurate and simple. It also ensures that similar-but-different
/// types don't get confused or create race conditions when deserialized.
pub struct ListenerInfo {
	pub evt_info: Option<EventInfo>,
	pub persistent: bool,
	pub listeners: Vec<(FilterCriteria, Sender<PortableEvent>)>,
	pub json_listeners: Vec<(FilterCriteria, Sender<PortableJSONEvent>)>,
	pub bincode_listeners: Vec<(FilterCriteria, Sender<PortableBincodeEvent>)>,
}

pub struct EventInfo {
	pub type_id: TypeId,
	pub deserializer: Box<dyn PortableMessageGenericDeserializer>,
}

impl ListenerInfo {
	pub fn new() -> Self {
		return ListenerInfo {
			evt_info: None,
			persistent: false,
			listeners: Vec::new(),
			json_listeners: Vec::new(),
			bincode_listeners: Vec::new(),
		};
	}
	pub fn new_declared<T: BidirectionalPortable>() -> Self {
		return ListenerInfo {
			evt_info: Some(EventInfo {
				type_id: TypeId::of::<T>(),
				deserializer: Box::new(PortableMessageDeserializer::<T>::new()),
			}),
			persistent: true,
			listeners: Vec::new(),
			json_listeners: Vec::new(),
			bincode_listeners: Vec::new(),
		}
	}

	pub fn declare<T: BidirectionalPortable>(&mut self) {
		self.evt_info = Some(EventInfo {
			type_id: TypeId::of::<T>(),
			deserializer: Box::new(PortableMessageDeserializer::<T>::new()),
		});
	}
}

fn relevant_listener<T: Sync + Send>(filter: &FilterCriteria, listeners: &[(FilterCriteria, Sender<PortableEventGeneric<T>>)]) -> bool {
	for listener in listeners {
		match listener.0 {
			FilterCriteria::None => { return true; },
			_ => {
				if listener.0 == *filter {
					return true;
				}
			}
		}
	}
	return false;
}

fn send_filtered<T: Sync + Send + 'static>(filter: &FilterCriteria, message: PortableEventGeneric<T>, listeners: &[(FilterCriteria, Sender<PortableEventGeneric<T>>)]) {
	for listener in listeners {
		if let FilterCriteria::None = listener.0 {
			let cloned_message = message.clone();
			listener.1.try_send(cloned_message).ok();
		} else if listener.0 == *filter {
			let cloned_message = message.clone();
			listener.1.try_send(cloned_message).ok();
		}
	}
}

async fn send_shutdown<T: Send + Sync>(listeners: &[(FilterCriteria, Sender<PortableEventGeneric<T>>)]) {
	for listener in listeners {
		listener.1.send(PortableEventGeneric::Shutdown).await.ok();
	}
}

#[portable]
#[derive(Eq, PartialEq)]
pub enum FilterCriteria {
	None,
	String(String),
	Uuid(Uuid),
}

#[portable]
pub enum DeclareEventError {
	AlreadyDeclared,
}

#[portable]
pub enum RegisterListenerError {
	EventClaimedAsType,
}

#[portable]
pub enum RegisterEncodedListenerError {
}
