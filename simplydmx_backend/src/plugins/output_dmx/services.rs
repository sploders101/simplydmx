use simplydmx_plugin_framework::*;
use uuid::Uuid;

use crate::utilities::serialized_data::SerializedData;

use super::interface::{DMXDriverDescription, DMXInterface, LinkUniverseError};

#[interpolate_service(
	"create_universe",
	"Create Universe",
	"Creates a new, unlinked universe for DMX output"
)]
impl CreateUniverse {
	#![inner_raw(DMXInterface)]
	pub fn new(interface: DMXInterface) -> Self {
		Self(interface)
	}

	#[service_main(
		("Name", "An arbitrary name to give the new universe"),
		("New universe ID", "Returns the UUID of the new universe", "DMX::universe-id"),
	)]
	async fn main(self, name: String) -> Uuid {
		return self.0.create_universe(name).await;
	}
}

#[interpolate_service(
	"delete_universe",
	"Delete Universe",
	"Deletes an existing universe, unlinking any associated lights or controllers"
)]
impl DeleteUniverse {
	#![inner_raw(DMXInterface)]
	pub fn new(interface: DMXInterface) -> Self {
		Self(interface)
	}

	#[service_main(
		("Universe ID", "The ID of the universe you would like to delete"),
	)]
	async fn main(self, universe_id: Uuid) -> () {
		return self.0.delete_universe(universe_id).await;
	}
}

#[interpolate_service(
	"link_universe",
	"Link Universe",
	"Links an existing universe to a DMX driver"
)]
impl LinkUniverse {
	#![inner_raw(DMXInterface)]
	pub fn new(interface: DMXInterface) -> Self {
		Self(interface)
	}

	#[service_main(
		("Universe ID", "The ID of the universe you would like to link"),
		("Driver ID", "The ID of the driver you would like to link the universe to"),
		("Form Data", "The form data, as described by `get_link_form()`"),
		("Result", "Result describing the error, if one occurred"),
	)]
	async fn main(
		self,
		universe_id: Uuid,
		driver: String,
		form_data: SerializedData,
	) -> Result<(), LinkUniverseError> {
		return self.0.link_universe(universe_id, driver, form_data).await;
	}
}

#[interpolate_service(
	"unlink_universe",
	"Unlink Universe",
	"Unlinks an existing universe from its driver"
)]
impl UnlinkUniverse {
	#![inner_raw(DMXInterface)]
	pub fn new(interface: DMXInterface) -> Self {
		Self(interface)
	}

	#[service_main(
		("Universe ID", "The ID of the universe you would like to unlink"),
	)]
	async fn main(self, universe_id: Uuid) {
		return self.0.unlink_universe(&universe_id).await;
	}
}

#[interpolate_service(
	"list_universes",
	"List Universes",
	"Lists the universes registered in the DMX driver"
)]
impl ListUniverses {
	#![inner_raw(DMXInterface)]
	pub fn new(interface: DMXInterface) -> Self {
		Self(interface)
	}

	#[service_main(
		("Universes", "An array of all universes registered with the DMX output driver"),
	)]
	async fn main(self) -> Vec<(Uuid, String)> {
		return self.0.list_universes().await;
	}
}

#[interpolate_service(
	"list_drivers",
	"List Drivers",
	"List the DMX device drivers registered with the DMX output driver"
)]
impl ListDrivers {
	#![inner_raw(DMXInterface)]
	pub fn new(interface: DMXInterface) -> Self {
		Self(interface)
	}

	#[service_main(
		("Drivers", "An array of all driver descriptions"),
	)]
	async fn main(self) -> Vec<DMXDriverDescription> {
		return self.0.list_drivers().await;
	}
}
