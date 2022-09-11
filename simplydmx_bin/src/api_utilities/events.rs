use std::collections::{HashMap, HashSet};
use futures::{FutureExt, select_biased};

use async_std::{
	channel::{
		self,
		Sender,
	},
	sync::{
		Arc,
		RwLock,
	},
};

use simplydmx_plugin_framework::*;

use super::JSONResponse;

#[derive(Hash, Eq, PartialEq)]
struct EventDescriptor(String, FilterCriteria);

enum RelayCommand {
	Stop,
}

struct EventJugglerInfo {
	sender: Sender<JSONResponse>,
	has_criteria_none: HashSet<String>,
	listeners: HashMap<EventDescriptor, Sender<RelayCommand>>,
}

#[derive(Clone)]
pub struct EventJuggler(PluginContext, Arc<RwLock<EventJugglerInfo>>);

impl EventJuggler {
	pub fn new(plugin_context: PluginContext, sender: Sender<JSONResponse>) -> Self {
		return EventJuggler(plugin_context, Arc::new(RwLock::new(EventJugglerInfo {
			sender,
			has_criteria_none: HashSet::new(),
			listeners: HashMap::new(),
		})));
	}

	pub async fn add_event_listener(&self, event_name: String, criteria: FilterCriteria) -> Result<(), RegisterEncodedListenerError> {
		let mut info = self.1.write().await;
		let descriptor = EventDescriptor(event_name.clone(), criteria.clone());
		if !info.listeners.contains_key(&descriptor) {
			let (sender, command_receiver) = channel::bounded(1);
			let event_receiver = self.0.on_json(event_name.clone(), criteria.clone()).await?;
			info.listeners.insert(descriptor, sender);

			let juggler = Self::clone(&self);

			let reusable_criteria = criteria.clone();
			let reusable_event_name = event_name.clone();
			self.0.spawn_volatile("API Event forwarder", async move {
				loop {
					select_biased! {
						event = event_receiver.recv().fuse() => {
							match event {
								Ok(PortableJSONEvent::Msg { data, criteria }) => {
									let juggler_info = juggler.1.read().await;
									let has_no_filter = juggler_info.has_criteria_none.contains(&reusable_event_name);
									if (reusable_criteria != FilterCriteria::None && has_no_filter)
										|| (reusable_criteria == FilterCriteria::None && !has_no_filter) {
										continue;
									}

									juggler_info.sender.send(JSONResponse::Event {
										name: reusable_event_name.clone(),
										criteria: FilterCriteria::clone(&criteria),
										data: serde_json::Value::clone(&data),
									}).await.ok();
								},
								Ok(PortableJSONEvent::Shutdown) => break,
								Err(_) => break,
							}
						},
						command = command_receiver.recv().fuse() => {
							match command {
								Ok(RelayCommand::Stop) => break,
								Err(_) => break,
							}
						},
					};
				}
			}).await;

			// If we don't have a filter, mark it in the hashmap to shut up the other receivers
			if let FilterCriteria::None = criteria {
				info.has_criteria_none.insert(event_name);
			}
		}

		return Ok(());
	}

	pub async fn remove_event_listener(&self, event_name: String, criteria: FilterCriteria) {
		let mut info = self.1.write().await;
		let descriptor = EventDescriptor(event_name.clone(), criteria.clone());

		if let Some(listener) = info.listeners.remove(&descriptor) {
			if criteria == FilterCriteria::None {
				info.has_criteria_none.remove(&event_name);
			}

			listener.send(RelayCommand::Stop).await.ok();
		}
	}
}
