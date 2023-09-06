//! This module provides services that can be linked to control surfaces

use std::{borrow::Cow, sync::Arc};

use crate::utilities::{forms::FormDescriptor, serialized_data::SerializedData};
use async_trait::async_trait;
use simplydmx_plugin_framework::*;
use thiserror::Error;

use super::types::Control;


#[portable]
#[derive(Debug, Clone, Error)]
pub enum ControllerLinkError {
	#[error("Bad form data.")]
	BadFormData,
	#[error("Bad form data. Details: {0}")]
	BadFormDataWithDetails(Cow<'static, str>),
	#[error("Error: {0}")]
	Other(Cow<'static, str>),
}

#[portable]
#[derive(Debug, Clone, Error)]
pub enum ControllerRestoreError {
	#[error("Bad form data.")]
	BadSaveData,
	#[error("Bad form data. Details: {0}")]
	BadSaveDataWithDetails(Cow<'static, str>),
	#[error("Error: {0}")]
	Other(Cow<'static, str>),
}

#[async_trait]
pub trait ControllerService {
	/// Gets a form for integrating a control group with a service
	///
	/// If prefilled is provided, the form data should be pre-filled with
	/// the data from `prefilled`. This data originates from the controller
	/// service link during editing.
	///
	/// `control_group` is an `Arc` containing a reference to the control
	/// group that is being edited.
	async fn get_form(
		&self,
		prefilled: Option<SerializedData>,
		control_group: Arc<Control>,
	) -> FormDescriptor;

	/// Links a control to the service
	async fn create_link(
		&self,
		form_data: SerializedData,
		control_group: Arc<Control>,
	) -> Result<Arc<dyn ControllerServiceLink + Send + Sync + 'static>, ControllerLinkError>;

	/// Loads a control from previously-saved data
	async fn load_from_save(
		&self,
		save_data: SerializedData,
		control_group: Arc<Control>,
	) -> Result<Arc<dyn ControllerServiceLink + Send + Sync + 'static>, ControllerRestoreError>;
}

#[async_trait]
pub trait ControllerServiceLink {
	async fn save(&self) -> SerializedData;
	async fn unlink(&mut self) {}
}
