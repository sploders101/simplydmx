use simplydmx_plugin_framework::*;

#[portable]
#[serde(transparent)]
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

	/// Creates a dropdown
	///
	/// Options can be sourced from either statically-inserted values or a TypeSpec source
	pub fn dropdown(mut self, label: impl Into<String>, id: impl Into<String>, options: FormItemOptionSource) -> Self {
		self.0.push(FormItem::Dropdown(FormDropdown { label: label.into(), id: id.into(), item_source: options }));
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
pub enum FormItem {
	Textbox(FormTextbox),
	Dropdown(FormDropdown),
	Section(FormSection),
	VerticalStack(Vec<FormItem>),
	HorizontalStack(Vec<FormItem>),
}

#[portable]
pub struct FormSection {
	label: String,
	form_items: Vec<FormItem>,
}

#[portable]
pub struct FormTextbox {
	label: String,
	id: String,
}

#[portable]
pub struct FormDropdown {
	/// The label to be displayed on the Dropdown
	label: String,
	/// The ID to use as this value's key in the response
	id: String,
	/// The method by which this dropdown should source its items
	item_source: FormItemOptionSource,
}

#[portable]
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
