# SimplyDMX Built-In Events

Listed below are all built-in events added by the SimplyDMX plugin framework. Any
events not listed here have been added by other plugins. While this is not enforced,
standard practice is to use one of the following conventions

* `plugin_id.event_id`
	* This standard should be used to emit events about something that happened internally
	to the emitting plugin, that other plugins may want to be aware of.
* `category_id.event_id`
	* This standard should be used when throwing out information that one of several
	plugins may want to handle.
	* Ex: `lighting_values.dmx` could be used to send values from the abstract layering
	plugin to the dmx handler.


## Core events

* `simplydmx.plugin_registered`
	* This event is called when a plugin is registered with the system
* `simplydmx.service_registered`
* `simplydmx.service_removed`
