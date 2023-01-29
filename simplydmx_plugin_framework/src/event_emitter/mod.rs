mod arc_portable;
mod event_receiver;
mod portable_message;

use std::{
	sync::Arc,
	collections::{HashMap, BTreeMap},
	any::TypeId,
};

use async_std::channel::{
	self,
	Sender,
	Receiver,
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
pub type PortableCborEvent = PortableEventGeneric<Vec<u8>>;
pub enum PortableEventGeneric<T: Sync + Send> {
	Msg {
		data: Arc<T>,
		criteria: Arc<FilterCriteria>,
	},
	Shutdown,
}

impl<T: Sync + Send> Clone for PortableEventGeneric<T> {
	fn clone(&self) -> Self {
		return match self {
			&PortableEventGeneric::Msg {
				ref data,
				ref criteria,
			} => PortableEventGeneric::Msg {
				data: Arc::clone(data),
				criteria: Arc::clone(criteria),
			},
			&PortableEventGeneric::Shutdown => PortableEventGeneric::Shutdown,
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
	shutdown_listeners: Vec<Sender<()>>,
}

impl EventEmitter {

	/// Creates a new EventEmitter.
	pub fn new() -> EventEmitter {
		return EventEmitter {
			listeners: HashMap::new(),
			shutdown_listeners: Vec::new(),
		};
	}

	/// Runs garbage collection for old receivers that are no longer active.
	/// This is kind of a scrappy implementation, but should work in the short-term
	/// to get an MVP running.
	fn gc(&mut self) {

		// Clear out shutdown listeners
		let mut i = 0;
		while i < self.shutdown_listeners.len() {
			if self.shutdown_listeners[i].receiver_count() == 0 {
				self.shutdown_listeners.remove(i);
			} else {
				i += 1;
			}
		}

		// Clear out listener Vecs
		let mut to_remove = Vec::new();
		for (event_id, listener_info) in self.listeners.iter_mut() {

			// PortableEvent Listeners
			i = 0;
			let listeners = &mut listener_info.listeners;
			while i < listeners.len() {
				if listeners[i].1.receiver_count() == 0 {
					listeners.remove(i);
				} else {
					i += 1;
				}
			}

			// JSON Listeners
			i = 0;
			let json_listeners = &mut listener_info.json_listeners;
			while i < json_listeners.len() {
				if json_listeners[i].1.receiver_count() == 0 {
					json_listeners.remove(i);
				} else {
					i += 1;
				}
			}

			// CBOR listeners
			i = 0;
			let cbor_listeners = &mut listener_info.cbor_listeners;
			while i < cbor_listeners.len() {
				if cbor_listeners[i].1.receiver_count() == 0 {
					cbor_listeners.remove(i);
				} else {
					i += 1;
				}
			}

			// Mark listener entry for removal if it's empty and wasn't declared
			if !listener_info.persistent
				&& listeners.len() == 0
				&& json_listeners.len() == 0
				&& cbor_listeners.len() == 0
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
	/// data (JSON and CBOR, for example) will be repeated verbatim on the bus for any listeners of the same protocol.
	///
	/// The type parameter is used to construct a generic deserializer used for translation.
	pub fn declare_event<T: BidirectionalPortable>(
		&mut self,
		event_name: String,
	) -> Result<(), DeclareEventError> {
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
	pub fn listen<T: BidirectionalPortable>(
		&mut self,
		event_name: String,
		filter: FilterCriteria,
	) -> Result<EventReceiver<T>, RegisterListenerError> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			self.listeners.insert(String::clone(&event_name), ListenerInfo::new());
		}

		// Unwrap ok here because we just created the entry
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

	/// Registers an event listeners on the bus of the given type.
	///
	/// Takes a closure to be called when an event occurs.
	pub fn on<T: BidirectionalPortable>(
		&mut self,
		event_name: String,
		filter: FilterCriteria,
		mut callback: impl FnMut(ArcPortable<T>, Arc<FilterCriteria>) -> () + Sync + Send + 'static,
	) -> Result<ListenerHandle, RegisterListenerError> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			self.listeners.insert(String::clone(&event_name), ListenerInfo::new());
		}

		// Unwrap ok here because we just created the entry
		let listener_info = self.listeners.get_mut(&event_name).unwrap();

		// Return error if types are incompatible. This is not required, but is done out of principle to
		// ensure event types are consistent, which will help later with automated self-documentation.
		if let Some(ref evt_info) = listener_info.evt_info {
			if evt_info.type_id != TypeId::of::<T>() {
				return Err(RegisterListenerError::EventClaimedAsType);
			}
		}

		let listener = Box::new(move |event: Arc<Box<dyn PortableMessage>>, criteria: Arc<FilterCriteria>| {
			if let Some(event) = ArcPortable::new(event) {
				callback(event, criteria);
			}
		});

		let id = Uuid::new_v4();
		listener_info.closure_listeners.insert(id, (filter, listener));
		return Ok(ListenerHandle(event_name, id));
	}

	pub fn off(&mut self, handle: ListenerHandle) {
		if let Some(listener_info) = self.listeners.get_mut(&handle.0) {
			listener_info.closure_listeners.remove(&handle.1);
		}
	}

	/// Registers a listener on the event bus that receives pre-encoded JSON events
	pub fn listen_json(&mut self, event_name: String, filter: FilterCriteria) -> Result<Receiver<PortableJSONEvent>, RegisterEncodedListenerError> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			self.listeners.insert(String::clone(&event_name), ListenerInfo::new());
		}

		let listener_info = self.listeners.get_mut(&event_name).unwrap();

		let (sender, receiver) = channel::unbounded();
		listener_info.json_listeners.push((filter, sender));
		return Ok(receiver);
	}

	/// Registers a listener on the bus that receives pre-encoded CBOR events
	pub fn on_cbor(&mut self, event_name: String, filter: FilterCriteria) -> Result<Receiver<PortableCborEvent>, RegisterEncodedListenerError> {
		self.gc();

		if !self.listeners.contains_key(&event_name) {
			self.listeners.insert(String::clone(&event_name), ListenerInfo::new());
		}

		let listener_info = self.listeners.get_mut(&event_name).unwrap();

		let (sender, receiver) = channel::unbounded();
		listener_info.cbor_listeners.push((filter, sender));
		return Ok(receiver);
	}

	pub fn on_shutdown(&mut self) -> Receiver<()> {
		self.gc();

		let (sender, receiver) = channel::unbounded();
		self.shutdown_listeners.push(sender);
		return receiver;
	}

	/// Sends an event on the bus. `T` gets cast to `Any`, boxed, wrapped in `Arc`,
	/// and sent to all registered listeners.
	pub async fn emit<T: BidirectionalPortable>(&mut self, event_name: String, filter: FilterCriteria, message: T) {
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {
			let filter = Arc::new(filter);

			// Re-broadcast JSON
			if relevant_listener(&filter, &listeners.json_listeners) {
				if let Ok(translated) = message.serialize_json() {
					send_filtered(&filter, PortableJSONEvent::Msg { data: Arc::new(translated), criteria: Arc::clone(&filter) }, &listeners.json_listeners);
				}
			}

			// Re-broadcast CBOR
			if relevant_listener(&filter, &listeners.cbor_listeners) {
				if let Ok(translated) = message.serialize_cbor() {
					send_filtered(&filter, PortableCborEvent::Msg { data: Arc::new(translated), criteria: Arc::clone(&filter) }, &listeners.cbor_listeners);
				}
			}

			let message: Arc<Box<dyn PortableMessage>> = Arc::new(Box::new(message));
			send_filtered(&filter, PortableEvent::Msg { data: Arc::clone(&message), criteria: Arc::clone(&filter) }, &listeners.listeners);
			send_filtered_closures(&filter, &message, &mut listeners.closure_listeners.values_mut());
		}

	}

	pub async fn emit_borrowed<T: BidirectionalPortable + Clone>(&mut self, event_name: String, filter: FilterCriteria, message: Arc<T>) {
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {
			let filter = Arc::new(filter);

			// Re-broadcast JSON
			if relevant_listener(&filter, &listeners.json_listeners) {
				if let Ok(translated) = message.serialize_json() {
					send_filtered(&filter, PortableJSONEvent::Msg { data: Arc::new(translated), criteria: Arc::clone(&filter) }, &listeners.json_listeners);
				}
			}

			// Re-broadcast CBOR
			if relevant_listener(&filter, &listeners.cbor_listeners) {
				if let Ok(translated) = message.serialize_cbor() {
					send_filtered(&filter, PortableCborEvent::Msg { data: Arc::new(translated), criteria: Arc::clone(&filter) }, &listeners.cbor_listeners);
				}
			}

			if relevant_listener(&filter, &listeners.listeners) {
				let message: Arc<Box<dyn PortableMessage>> = Arc::new(Box::new(T::clone(&message)));
				send_filtered(&filter, PortableEvent::Msg { data: Arc::clone(&message), criteria: Arc::clone(&filter) }, &listeners.listeners);
				send_filtered_closures(&filter, &message, &mut listeners.closure_listeners.values_mut());
			}

		}

	}

	/// Emits a JSON value to the bus, deserializing for listeners of other formats if
	/// necessary/possible. It will always be repeated to JSON listeners, but will silently
	/// fail to repeat on listeners of other protocols if deserialization fails
	pub async fn emit_json(&mut self, event_name: String, filter: FilterCriteria, message: serde_json::Value) {
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {
			let filter = Arc::new(filter);

			let deserialized: Option<Box<dyn PortableMessage>> = if
				relevant_listener(&filter, &listeners.listeners)
				|| relevant_listener(&filter, &listeners.cbor_listeners)
			{ if let Some(ref evt_info) = listeners.evt_info { evt_info.deserializer.deserialize_json(message.clone()).ok() } else { None } }
			else { None };

			// Re-broadcast JSON
			if relevant_listener(&filter, &listeners.json_listeners) {
				send_filtered(&filter, PortableJSONEvent::Msg { data: Arc::new(message), criteria: Arc::clone(&filter) }, &listeners.json_listeners);
			}

			// Re-broadcast CBOR
			if relevant_listener(&filter, &listeners.cbor_listeners) {
				if let Ok(translated) = deserialized.as_ref().unwrap().serialize_cbor() {
					send_filtered(&filter, PortableCborEvent::Msg { data: Arc::new(translated), criteria: Arc::clone(&filter) }, &listeners.cbor_listeners);
				}
			}

			// Deserialized
			if let Some(deserialized) = deserialized {
				let message: Arc<Box<dyn PortableMessage>> = Arc::new(deserialized);
				send_filtered(&filter, PortableEvent::Msg { data: Arc::clone(&message), criteria: Arc::clone(&filter) }, &listeners.listeners);
				send_filtered_closures(&filter, &message, &mut listeners.closure_listeners.values_mut());
			}

		}
	}

	/// Emits a CBOR value to the bus, deserializing for listeners of other formats if
	/// necessary/possible. It will always be repeated to CBOR listeners, but will silently
	/// fail to repeat on listeners of other protocols if deserialization fails
	pub async fn emit_cbor(&mut self, event_name: String, filter: FilterCriteria, message: Vec<u8>) {
		self.gc();

		if let Some(listeners) = self.listeners.get_mut(&event_name) {
			let filter = Arc::new(filter);

			let deserialized: Option<Box<dyn PortableMessage>> = if
				relevant_listener(&filter, &listeners.listeners)
				|| relevant_listener(&filter, &listeners.json_listeners)
			{ if let Some(ref evt_info) = listeners.evt_info { evt_info.deserializer.deserialize_cbor(&message).ok() } else { None }}
			else { None };

			// Re-broadcast JSON
			if relevant_listener(&filter, &listeners.json_listeners) {
				if let Ok(translated) = deserialized.as_ref().unwrap().serialize_json() {
					send_filtered(&filter, PortableJSONEvent::Msg { data: Arc::new(translated), criteria: Arc::clone(&filter) }, &listeners.json_listeners);
				}
			}

			// Re-broadcast CBOR
			if relevant_listener(&filter, &listeners.json_listeners) {
				send_filtered(&filter, PortableCborEvent::Msg { data: Arc::new(message), criteria: Arc::clone(&filter) }, &listeners.cbor_listeners);
			}

			// Deserialized
			if let Some(deserialized) = deserialized {
				let message: Arc<Box<dyn PortableMessage>> = Arc::new(deserialized);
				send_filtered(&filter, PortableEvent::Msg { data: Arc::clone(&message), criteria: Arc::clone(&filter) }, &listeners.listeners);
				send_filtered_closures(&filter, &message, &mut listeners.closure_listeners.values_mut());
			}

		}
	}

	pub async fn send_shutdown(&mut self) {
		self.gc();

		for shutdown_listener in self.shutdown_listeners.iter() {
			shutdown_listener.send(()).await.ok();
		}
		for listener_group in self.listeners.values() {
			send_shutdown(&listener_group.listeners).await;
			send_shutdown(&listener_group.json_listeners).await;
			send_shutdown(&listener_group.cbor_listeners).await;
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
	pub closure_listeners: BTreeMap<
		Uuid,
		(
			FilterCriteria,
			Box<dyn FnMut(Arc<Box<dyn PortableMessage>>, Arc<FilterCriteria>) -> () + Sync + Send + 'static>,
		),
	>,
	pub json_listeners: Vec<(FilterCriteria, Sender<PortableJSONEvent>)>,
	pub cbor_listeners: Vec<(FilterCriteria, Sender<PortableCborEvent>)>,
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
			closure_listeners: BTreeMap::new(),
			json_listeners: Vec::new(),
			cbor_listeners: Vec::new(),
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
			closure_listeners: BTreeMap::new(),
			json_listeners: Vec::new(),
			cbor_listeners: Vec::new(),
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

fn send_filtered_closures(
	filter: &Arc<FilterCriteria>,
	message: &Arc<Box<dyn PortableMessage>>,
	listeners: &mut dyn Iterator<
		Item = &mut (
			FilterCriteria,
			Box<
				dyn FnMut(Arc<Box<dyn PortableMessage>>, Arc<FilterCriteria>) -> ()
					+ Sync
					+ Send
					+ 'static,
			>,
		),
	>,
) {
	for listener in listeners {
		if let FilterCriteria::None = listener.0 {
			let cloned_message = Arc::clone(message);
			listener.1(cloned_message, Arc::clone(filter));
		} else if listener.0 == **filter {
			let cloned_message = Arc::clone(message);
			listener.1(cloned_message, Arc::clone(filter));
		}
	}
}


async fn send_shutdown<T: Send + Sync>(listeners: &[(FilterCriteria, Sender<PortableEventGeneric<T>>)]) {
	for listener in listeners {
		listener.1.send(PortableEventGeneric::Shutdown).await.ok();
	}
}

#[must_use]
/// A handle to a listener. This can be used to clean up the listener later
/// by calling the `off` function.
pub struct ListenerHandle(String, Uuid);
impl ListenerHandle {
	/// This function drops the handle to the event, causing it to run for the
	/// duration of the `EventEmitter`'s lifetime. Avoid calling this outside
	/// of a setup function.
	pub fn drop(self) {}
}

#[portable]
#[derive(Hash, Eq, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum FilterCriteria {
	None,
	String(String),
	Uuid(Uuid),
}

#[portable]
#[serde(tag = "type")]
pub enum DeclareEventError {
	AlreadyDeclared,
}

#[portable]
#[serde(tag = "type")]
pub enum RegisterListenerError {
	EventClaimedAsType,
}

#[portable]
#[serde(tag = "type")]
pub enum RegisterEncodedListenerError {
}
