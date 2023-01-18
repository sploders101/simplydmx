# Events

## Mixer

* Layer bin outputs
	* This is emitted whenever a layer bin has been blended and the output is ready for consumption.
	* `mixer.layer_bin_output`: `(Uuid, Arc<FullMixerOutput>)`
	* `mixer.layer_bin_output.${uuid.to_string()}`: `Arc<FullMixerOutput>`

* Final mixer output
	* This is emitted after the layers have been blended together and the output is ready to be
	  serialized and sent to lights
	* `mixer.final_output`: `Arc<FullMixerOutput>`

* Opacity values
	* These are emitted whenever an opacity changes, and they are useful for things like HID integrations.
	* `mixer.submaster_opacity`: `u16`
	* `mixer.layer_bin_opacity`: `u16`

* Submaster content changes
	* These are emitted whenever the content of a submaster changes, and are useful for updating the UI.
	* `mixer.submaster_content`: `SubmasterDelta`
