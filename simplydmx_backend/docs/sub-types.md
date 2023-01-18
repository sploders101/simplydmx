# Sub-types

Plugins can implement sub-types that allow for complex, auto-discoverable values that provide a
richer, easier user experience.

One example of this is the use of a service that requires a fixture ID.
The user should not be burdened with finding the unique identifier of a fixture, but a frontend can
search the plugin framework for a data provider using the requested sub-type ID and get potential options
for the user to select from a list.

Sub-types that have no provider may also be used to indicate value ranges, hint to human interface plugins
like MIDI where user-input values should go, etc.


## Used sub-type values

This section will document sub-type values implemented by each plugin and document whether a provider has
been or will be implemented.


### Mixer

* `mixer::layer_id`
	* Identifies a layer or submaster
* `mixer::layer_bin_id`
	* Identifies a layer bin (used for blind mode)
