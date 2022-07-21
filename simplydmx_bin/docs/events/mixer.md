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
