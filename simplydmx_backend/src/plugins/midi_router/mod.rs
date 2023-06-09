use midir::{MidiInput, MidiOutput, MidiInputPort, MidiOutputPort};
use midly::live::LiveEvent;
use rustc_hash::FxHashMap;
pub use simplydmx_plugin_framework::*;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

pub type MidiCallback = Box<dyn FnMut(LiveEvent) -> ()>;

pub enum MidiPort {
	Input(MidiInputPort),
	Output(MidiOutputPort),
}

pub struct MidiRouterInner {
	sinks: RwLock<FxHashMap<Uuid, Mutex<MidiCallback>>>,
	midir_source_client: MidiInput,
	midir_sink_client: MidiOutput,
	midir_known_ports: RwLock<FxHashMap<Uuid, MidiPort>>,
	// midir_sources: RwLock<FxHashMap<Uuid, Mutex<_>>>,
}

/// The MIDI router is a MIDI interface aggregator. It is responsible for
/// abstracting platform, protocol, and application-specific protocol
/// details. The MIDI router is the central connecting point for all
/// MIDI-based connections such as remote IAC connectors, midir, remote
/// clients, etc.
///
/// This interface also allows quick, user-friendly mapping of virtual
/// interfaces to physical ones.
pub struct MidiRouterInterface {
	plugin: PluginContext,
	inner: Arc<MidiRouterInner>,
}
impl MidiRouterInterface {
	pub async fn init(plugin_framework: &PluginManager) -> anyhow::Result<Self> {
		let plugin = plugin_framework
			.register_plugin("midi_router", "MIDI Router")
			.await
			.unwrap();

		return Ok(MidiRouterInterface {
			plugin,
			inner: Arc::new(MidiRouterInner {
				sinks: RwLock::new(FxHashMap::default()),
				midir_source_client: MidiInput::new("SimplyDMX")?,
				midir_sink_client: MidiOutput::new("SimplyDMX")?,
				midir_known_ports: RwLock::new(FxHashMap::default()),
				// midir_sources: RwLock::new(FxHashMap::default()),
			}),
		});
	}

	/// Poll midi driver for ports and synchronize with internal UUID mappings
	pub async fn poll_sources(&self) {
		let mut source_ports = self.inner.midir_source_client.ports();
		let mut sink_ports = self.inner.midir_sink_client.ports();
		let mut new_known_ports = self.inner.midir_known_ports.write().await;
		let old_known_ports = std::mem::take(&mut *new_known_ports);

		// Add any old ports that still exist back into new_known_ports
		// and remove them from source_ports and sink_ports, leaving only
		// new ports.
		for (id, port) in old_known_ports {
			match port {
				MidiPort::Input(port) => {
					let mut found_port = false;
					// Remove any ports that match this one from source_ports and
					// record if we found one.
					source_ports.retain(|this_port| {
						let result = this_port == &port;
						if result {
							found_port = true;
						}
						!result
					});
					// If we found the old port in the new list, insert it with the
					// existing UUID in the new HashMap
					if found_port {
						new_known_ports.insert(id, MidiPort::Input(port));
					}
				}
				MidiPort::Output(port) => {
					let mut found_port = false;
					// Remove any ports that match this one from sink_ports and
					// record if we found one.
					sink_ports.retain(|this_port| {
						let result = this_port == &port;
						if result {
							found_port = true;
						}
						!result
					});
					// If we found the old port in the new list, insert it with the
					// existing UUID in the new HashMap
					if found_port {
						new_known_ports.insert(id, MidiPort::Output(port));
					}
				}
			}
		}

		// Generate new UUIDs for any ports we didn't recognize
		for port in source_ports {
			new_known_ports.insert(Uuid::new_v4(), MidiPort::Input(port));
		}
		for port in sink_ports {
			new_known_ports.insert(Uuid::new_v4(), MidiPort::Output(port));
		}
	}
}
