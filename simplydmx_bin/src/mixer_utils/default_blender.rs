use super::{
	state::{
		FullMixerOutput,
		BlendingData,
		SnapData,
		BlenderValue,
		BlendingScheme,
		SubmasterData,
	},
	data_sources::LayerDataSourcesLocked,
};

pub async fn blend_layer(cumulative_layer: &mut FullMixerOutput, data_sources: &LayerDataSourcesLocked, opacity: u16, submaster: &SubmasterData) {
	let blending_data = data_sources.blending_data();

	// For each light within the submaster
	for (fixture_id, fixture_data) in submaster.iter() {
		// Get the fixture blending instructions and layer
		if let (
			Some(fixture_blending_data),
			Some(cumulative_fixture),
		) = (
			blending_data.get(fixture_id),
			cumulative_layer.get_mut(fixture_id),
		) {
			// For each property within the light
			for (attribute_id, attribute_value) in fixture_data.iter() {
				// Get blending instructions
				if let (
					Some(blending_data),
					Some(cumulative_attribute),
				) = (
					fixture_blending_data.get(attribute_id),
					cumulative_fixture.get(attribute_id),
				) {
					// Blend the value
					match attribute_value {
						BlenderValue::None => {},
						BlenderValue::Offset(attribute_value) => {
							// Blending scheme not used for offset values
							let submaster_opacity = opacity_modifier(opacity.clone(), blending_data);
							let faded_value = f64::from(attribute_value.clone()) * (f64::from(submaster_opacity) / f64::from(u16::MAX));
							let new_value = f64::from(cumulative_attribute.clone()) + faded_value;
							cumulative_fixture.insert(attribute_id.clone(), new_value.clamp(u16::MIN as f64, u16::MAX as f64).round() as u16);
						},
						BlenderValue::Static(attribute_value) => {
							match blending_data.scheme {
								BlendingScheme::HTP => {
									let submaster_opacity = opacity_modifier(opacity.clone(), blending_data);
									let faded_value = f64::from(attribute_value.clone()) * (f64::from(submaster_opacity) / f64::from(u16::MAX));
									let faded_value = faded_value.clamp(0f64, 65535f64).round() as u16;
									if &faded_value > cumulative_attribute {
										cumulative_fixture.insert(attribute_id.clone(), faded_value);
									}
								},
								BlendingScheme::LTP => {
									// Avoid simultaneous mutable & immutable borrow by cloning upfront
									let cumulative_attribute = cumulative_attribute.clone();
									// Blend and insert faded value
									cumulative_fixture.insert(
										attribute_id.clone(),
										blend_ltp(
											cumulative_attribute,
											attribute_value.clone(),
											opacity.clone()
										)
									);
								}
							};
						},
					};
				}
			}
		}
	}
}

/// Aids in snapping, so an attribute that uses the "snapping" feature doesn't blend.
///
/// If the opacity is past the snapping threshold, this function returns maximum opacity.
/// If it is below the threshold, it returns 0.
fn opacity_modifier(submaster_opacity: u16, blending_data: &BlendingData) -> u16 {
	return match blending_data.snap {
		SnapData::NoSnap => submaster_opacity,
		SnapData::SnapAt(fulcrum) => if submaster_opacity > fulcrum { u16::MAX } else { u16::MIN },
	};
}

/// Blend two values as LTP, taking opacity into account
fn blend_ltp(current_value: u16, new_value: u16, opacity: u16) -> u16 {
	let start = f64::from(current_value);
	let end = f64::from(new_value);
	let opacity = f64::from(opacity);
	let max_opacity = f64::from(u16::MAX);
	let faded_value: f64 = (end - start) * (opacity / max_opacity) + start;
	let faded_value = faded_value.clamp(0f64, 65535f64).round() as u16;
	return faded_value;
}
