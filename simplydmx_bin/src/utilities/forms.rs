use simplydmx_plugin_framework::*;

#[portable]
#[serde(transparent)]
/// Describes a form-style UI using a frontend-agnostic generic data structure
pub struct FormDescriptor(Vec<FormItem>);
impl FormDescriptor {
	pub fn new() -> FormDescriptor {
		return FormDescriptor(Vec::new());
	}

	/// Creates a textbox
	pub fn textbox(mut self, label: impl Into<String>, id: impl Into<String>) -> Self {
		self.0.push(FormItem::Textbox(FormTextbox { label: label.into(), id: id.into() }));
		return self;
	}

	/// Creates a number input
	pub fn number(mut self, label: impl Into<String>, id: impl Into<String>) -> Self {
		self.0.push(FormItem::Number(FormNumber { label: label.into(), id: id.into() }));
		return self;
	}

	/// Creates a dropdown with static options
	pub fn dropdown_static(mut self, label: impl Into<String>, id: impl Into<String>, options: impl Fn(OptionsBuilder) -> OptionsBuilder) -> Self {
		self.0.push(FormItem::Dropdown(FormDropdown { label: label.into(), id: id.into(), item_source: options(OptionsBuilder(Vec::new())).into() }));
		return self;
	}

	/// Creates a dropdown with options sourced from a provider registered in the plugin framework
	pub fn dropdown_dynamic(mut self, label: impl Into<String>, id: impl Into<String>, typespec_id: impl Into<String>) -> Self {
		self.0.push(FormItem::Dropdown(FormDropdown {
			label: label.into(),
			id: id.into(),
			item_source: FormItemOptionSource::TypeSpec { typespec_id: typespec_id.into() },
		}));
		return self;
	}

	/// Creates a labeled section for the form.
	///
	/// `builder` can be used to easily construct the contents of the section.
	///
	/// Example:
	/// ```rust
	/// FormDescriptor::new()
	///     .section(|form| form
	///         .textbox("My Textbox", "myTextbox")
	///         .textbox("Another textbox", "anotherTextbox")
	///     )
	/// ```
	pub fn section(mut self, label: impl Into<String>, builder: impl FnOnce(FormDescriptor) -> FormDescriptor) -> Self {
		self.0.push(FormItem::Section(FormSection { label: label.into(), form_items: builder(FormDescriptor::new()).0 }));
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
		self.0.push(FormItem::VerticalStack(builder(FormDescriptor::new()).0));
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
		self.0.push(FormItem::HorizontalStack(builder(FormDescriptor::new()).0));
		return self;
	}
}

#[portable]
/// Describes a form element
pub enum FormItem {
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
	label: String,
	form_items: Vec<FormItem>,
}

#[portable]
/// Describes a textbox as part of a form
pub struct FormTextbox {
	label: String,
	id: String,
}

#[portable]
/// Describes a number input as part of a form
pub struct FormNumber {
	label: String,
	id: String,
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
}

#[portable]
/// Describes a source for dropdown/autocomplete options
pub enum FormItemOptionSource {

	/// Use a static set of values as the dropdown options
	Static {
		values: Vec<DropdownOptionJSON>,
	},

	/// Use a type specifier to source dropdown options. These are a plugin
	/// framework construct that can be queried through the JSON API.
	TypeSpec{
		typespec_id: String
	},

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
		return FormItemOptionSource::Static {
			values: self.0,
		};
	}
}
