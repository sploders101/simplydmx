mod dmxsource_controller;

use std::{
	collections::HashMap,
	sync::Arc,
};

use async_std::{sync::RwLock, channel::Sender};

use simplydmx_plugin_framework::*;
use uuid::Uuid;

use self::dmxsource_controller::{E131Command, initialize_controller};

pub struct E131State {
	controller: Sender<E131Command>,
	universes: HashMap<Uuid, u16>,
}
impl E131State {
	pub fn new() -> Self {
		return E131State {
			controller: initialize_controller(),
			universes: HashMap::new(),
		}
	}
}

#[interpolate_service(
	"create_universe",
	"Create Universe",
	"Creates an E.131/sACN universe for transmission of DMX over the network",
)]
impl CreateE131Universe {

	#![inner_raw(PluginContext, Arc::<RwLock::<E131State>>)]

	#[service_main(
		("Internal Universe ID", "The universe UUID used internally for this output", "output::int-universe-uuid"),
		("External Universe ID", "The universe ID used on any associated sACN lights or gateways.", "output::user-sourced"),
		("Result", "The result of the request", "output::result-static"),
	)]
	async fn main(self, int_id: Uuid, ext_id: u16) -> Result<(), &'static str> {
		let mut ctx = self.1.write().await;

		if let Some(_) = ctx.universes.values().find(|universe_id| **universe_id == ext_id) {
			return Err("This external universe ID is taken");
		}

		if ctx.universes.len() == 0 {
			if let Err(_) = ctx.controller.send(E131Command::CreateOutput).await {
				log_error!(self.0, "The E.131 controller exited early!");
			}
		}
		ctx.universes.insert(int_id, ext_id);

		return Ok(());
	}
}


#[interpolate_service(
	"destroy_universe",
	"Destroy Universe",
	"Destroys an E.131/sACN universe, signaling end of transmission",
)]
impl DestroyE131Universe {

	#![inner_raw(PluginContext, Arc::<RwLock::<E131State>>)]

	#[service_main(
		("Internal Universe ID", "The universe UUID used internally for this output", "output::int-universe-uuid"),
	)]
	async fn main(self, int_id: Uuid) -> () {
		let mut ctx = self.1.write().await;

		if let Some(ext_id) = ctx.universes.remove(&int_id) {
			if let Err(_) = ctx.controller.send(E131Command::TerminateUniverse(ext_id)).await {
				log_error!(self.0, "The E.131 controller exited early!");
			}
			if ctx.universes.len() == 0 {
				if let Err(_) = ctx.controller.send(E131Command::DestroyOutput).await {
					log_error!(self.0, "The E.131 controller exited early!");
				}
			}
		}
	}
}

#[portable]
#[serde(untagged)]
pub enum DMXFrame {
	#[serde(skip)]
	FixedLength([u8; 512]),
	Vec(Vec<u8>),
}
impl Into<[u8; 512]> for DMXFrame {
	fn into(self) -> [u8; 512] {
		return match self {
			Self::FixedLength(fixed_length) => fixed_length,
			Self::Vec(vec_frame) => {
				let mut new_frame = [0u8; 512];
				for (i, item) in vec_frame.into_iter().enumerate() {
					if i <= 512 {
						new_frame[i] = item;
					}
				}
				new_frame
			},
		};
	}
}

#[interpolate_service(
	"send_frame",
	"Send DMX frame",
	"Sends a DMX frame for the specified universe "
)]
impl SendOutput {

	#![inner_raw(PluginContext, Arc::<RwLock::<E131State>>)]

	#[service_main(
		("Internal Universe ID", "The universe UUID used internally for this output", "output::int-universe-id"),
		("DMX Data", "The DMX data, pre-serialized and ready to send", "output::dmx-data"),
	)]
	async fn main(self, int_id: Uuid, data: DMXFrame) -> () {
		let ctx = self.1.read().await;
		if let Some(ext_universe) = ctx.universes.get(&int_id) {
			if let Err(_) = ctx.controller.send(E131Command::SendOutput(*ext_universe, data.into())).await {
				log_error!(self.0, "The E.131 controller exited early!");
			}
		}
	}

}
