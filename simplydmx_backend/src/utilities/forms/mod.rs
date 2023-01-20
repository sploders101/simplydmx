mod interactivity;
mod validation;

use simplydmx_plugin_framework::*;

pub use self::interactivity::InteractiveDescription;
pub use self::validation::{ImplicitNumberValidation, NumberValidation};

#[portable]
#[serde(transparent)]
/// Describes a form-style UI using a frontend-agnostic generic data structure
pub struct FormDescriptor(Vec<FormItem>);
impl FormDescriptor {
	pub fn new() -> FormDescriptor {
		return FormDescriptor(Vec::new());
	}

	/// Creates a hide-able collection of form items
	pub fn dynamic(
		mut self,
		conditions: InteractiveDescription,
		builder: impl FnOnce(FormDescriptor) -> FormDescriptor,
	) -> Self {
		self.0.push(FormItem::Dynamic(
			conditions,
			builder(FormDescriptor::new()).0,
		));
		return self;
	}

	/// Creates a textbox
	pub fn textbox(mut self, label: impl Into<String>, id: impl Into<String>) -> Self {
		self.0.push(FormItem::Textbox(FormTextbox {
			label: label.into(),
			id: id.into(),
			value: None,
		}));
		return self;
	}

	/// Creates a textbox with a pre-filled value
	pub fn textbox_prefilled(
		mut self,
		label: impl Into<String>,
		id: impl Into<String>,
		value: impl Into<String>,
	) -> Self {
		self.0.push(FormItem::Textbox(FormTextbox {
			label: label.into(),
			id: id.into(),
			value: Some(value.into()),
		}));
		return self;
	}

	/// Creates a number input
	pub fn number(
		mut self,
		label: impl Into<String>,
		id: impl Into<String>,
		validation: NumberValidation,
	) -> Self {
		self.0.push(FormItem::Number(FormNumber {
			label: label.into(),
			id: id.into(),
			validation,
			value: None,
		}));
		return self;
	}

	/// Creates a number input with a pre-filled value
	pub fn number_prefilled(
		mut self,
		label: impl Into<String>,
		id: impl Into<String>,
		validation: NumberValidation,
		value: f64,
	) -> Self {
		self.0.push(FormItem::Number(FormNumber {
			label: label.into(),
			id: id.into(),
			validation,
			value: Some(value),
		}));
		return self;
	}

	/// Creates a dropdown with static options
	pub fn dropdown_static(
		mut self,
		label: impl Into<String>,
		id: impl Into<String>,
		options: impl FnOnce(OptionsBuilder) -> OptionsBuilder,
	) -> Self {
		self.0.push(FormItem::Dropdown(FormDropdown {
			label: label.into(),
			id: id.into(),
			item_source: options(OptionsBuilder(Vec::new())).into(),
			value: serde_json::Value::Null,
		}));
		return self;
	}

	/// Creates a pre-filled dropdown with static options
	pub fn dropdown_static_prefilled(
		mut self,
		label: impl Into<String>,
		id: impl Into<String>,
		options: impl FnOnce(OptionsBuilder) -> OptionsBuilder,
		value: serde_json::Value,
	) -> Self {
		self.0.push(FormItem::Dropdown(FormDropdown {
			label: label.into(),
			id: id.into(),
			item_source: options(OptionsBuilder(Vec::new())).into(),
			value,
		}));
		return self;
	}

	/// Creates a dropdown with options sourced from a provider registered in the plugin framework
	pub fn dropdown_dynamic(
		mut self,
		label: impl Into<String>,
		id: impl Into<String>,
		typespec_id: impl Into<String>,
	) -> Self {
		self.0.push(FormItem::Dropdown(FormDropdown {
			label: label.into(),
			id: id.into(),
			item_source: FormItemOptionSource::TypeSpec {
				typespec_id: typespec_id.into(),
			},
			value: serde_json::Value::Null,
		}));
		return self;
	}

	/// Creates a pre-filled dropdown with options sourced from a provider registered in the plugin framework
	pub fn dropdown_dynamic_prefilled(
		mut self,
		label: impl Into<String>,
		id: impl Into<String>,
		typespec_id: impl Into<String>,
		value: serde_json::Value,
	) -> Self {
		self.0.push(FormItem::Dropdown(FormDropdown {
			label: label.into(),
			id: id.into(),
			item_source: FormItemOptionSource::TypeSpec {
				typespec_id: typespec_id.into(),
			},
			value,
		}));
		return self;
	}

	/// Creates a labeled section for the form.
	///
	/// `builder` can be used to easily construct the contents of the section.
	///
	/// If more than one item is given in the builder, a VerticalStack is created automatically.
	///
	/// Example:
	/// ```rust
	/// FormDescriptor::new()
	///     .section(|form| form
	///         .textbox("My Textbox", "myTextbox")
	///         .textbox("Another textbox", "anotherTextbox")
	///     )
	/// ```
	pub fn section(
		mut self,
		label: impl Into<String>,
		builder: impl FnOnce(FormDescriptor) -> FormDescriptor,
	) -> Self {
		// Build item
		let mut item = builder(FormDescriptor::new()).0;
		let item = if item.len() == 0 {
			return self;
		} else if item.len() == 1 {
			item.pop().unwrap()
		} else {
			FormItem::VerticalStack(item)
		};

		// Add section to FormDescriptor
		self.0.push(FormItem::Section(FormSection {
			label: label.into(),
			form_item: Box::new(item),
		}));
		return self;
	}

	/// Creates a vertical stack.
	///
	/// `builder` can be used to easily construct the contents of the section.
	///
	/// Example:
	/// ```rust
	/// FormDescriptor::new()
	///     .vertical(|form| form
	///         .textbox("My Textbox", "myTextbox")
	///         .textbox("Another textbox", "anotherTextbox")
	///     )
	/// ```
	pub fn vertical(mut self, builder: impl FnOnce(FormDescriptor) -> FormDescriptor) -> Self {
		self.0
			.push(FormItem::VerticalStack(builder(FormDescriptor::new()).0));
		return self;
	}

	/// Creates a horizontal stack.
	///
	/// `builder` can be used to easily construct the contents of the section.
	///
	/// Example:
	/// ```rust
	/// FormDescriptor::new()
	///     .horizontal(|form| form
	///         .textbox("My Textbox", "myTextbox")
	///         .textbox("Another textbox", "anotherTextbox")
	///     )
	/// ```
	pub fn horizontal(mut self, builder: impl FnOnce(FormDescriptor) -> FormDescriptor) -> Self {
		self.0
			.push(FormItem::HorizontalStack(builder(FormDescriptor::new()).0));
		return self;
	}

	/// Builds the form data into its final representation
	///
	/// Currently a no-op, but will be changed later to flatten into a single `FormItem` instance
	pub fn build(self) -> Self {
		return self;
	}
}

#[portable]
/// Describes a form element
pub enum FormItem {
	Dynamic(InteractiveDescription, Vec<FormItem>),
	Textbox(FormTextbox),
	Number(FormNumber),
	Dropdown(FormDropdown),
	Section(FormSection),
	VerticalStack(Vec<FormItem>),
	HorizontalStack(Vec<FormItem>),
}

#[portable]
/// Describes a visual container for form elements
pub struct FormSection {
	/// The label to give this section of the form
	label: String,
	/// The item that should be rendered within the form section
	form_item: Box<FormItem>,
}

#[portable]
/// Describes a textbox as part of a form
pub struct FormTextbox {
	/// The label to give this textbox
	label: String,
	/// The ID to give the field (ex: `formData[FormNumber::id] = FormNumber::value`)
	id: String,
	/// The value to give this textbox upon initial creation of the form
	value: Option<String>,
}

#[portable]
/// Describes a number input as part of a form
pub struct FormNumber {
	/// The label to give this field
	label: String,
	/// The ID to give the field (ex: `formData[FormNumber::id] = FormNumber::value`)
	id: String,
	/// The validation criteria for this number field
	validation: NumberValidation,
	/// The value to give this number field upon initial creation of the form
	value: Option<f64>,
}

#[portable]
/// Describes a dropdown component as part of a form
pub struct FormDropdown {
	/// The label to be displayed on the Dropdown
	label: String,
	/// The ID to use as this value's key in the response
	id: String,
	/// The method by which this dropdown should source its items
	item_source: FormItemOptionSource,
	/// The value to give this dropdown upon initial creation of the form
	value: serde_json::Value,
}

#[portable]
/// Describes a source for dropdown/autocomplete options
pub enum FormItemOptionSource {
	/// Use a static set of values as the dropdown options
	Static { values: Vec<DropdownOptionJSON> },

	/// Use a type specifier to source dropdown options. These are a plugin
	/// framework construct that can be queried through the JSON API.
	TypeSpec { typespec_id: String },
}

pub struct OptionsBuilder(Vec<DropdownOptionJSON>);
impl OptionsBuilder {
	pub fn add_item(mut self, name: impl Into<String>, value: impl Into<String>) {
		self.0.push(DropdownOptionJSON {
			name: name.into(),
			description: None,
			value: value.into().serialize_json().unwrap(),
		});
	}
}
impl Into<FormItemOptionSource> for OptionsBuilder {
	fn into(self) -> FormItemOptionSource {
		return FormItemOptionSource::Static { values: self.0 };
	}
}
