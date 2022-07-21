# Plugin Design

In order to create consistency in the way plugins are created and used, their must follow a
set of standards as defined here. These standards only serve as a rule of thumb, and are not
intended to restrict plugin developers in any way, but rather create consistency across plugins
so they can all communicate in a similar manner.


## File Structure

The file structure of a plugin should be as follows:

* `plugins`
	* `plugin_name`
		* `mod.rs`
		* `state.rs`
		* `services.rs`
		*
