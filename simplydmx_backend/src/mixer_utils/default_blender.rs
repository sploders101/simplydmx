use rayon::prelude::*;
use uuid::Uuid;

use super::{
	data_sources::LayerDataSourcesLocked,
	state::{
		AbstractLayerLight, BlenderValue, BlendingData, BlendingScheme, FixtureMixerOutput,
		FullMixerOutput, SnapData, SubmasterData,
	},
};

pub fn blend_fixture(
	fixture_id: &Uuid,
	cumulative_fixture: &mut FixtureMixerOutput,
	data_sources: &LayerDataSourcesLocked,
	opacity: u16,
	submaster: &AbstractLayerLight,
) {
	let blending_data = data_sources.blending_data();

	// Get the fixture blending instructions and layer
	if let Some(fixture_blending_data) = blending_data.get(fixture_id) {
		for (attribute_id, cumulative_attribute) in cumulative_fixture.iter_mut() {
			// Get blending instructions
			if let (Some(blending_data), Some(attribute_value)) = (
				fixture_blending_data.get(attribute_id),
				submaster.get(attribute_id),
			) {
				// Blend the value
				match attribute_value {
					BlenderValue::None => {}
					BlenderValue::Offset(attribute_value) => {
						// Blending scheme not used for offset values
						let submaster_opacity = opacity_modifier(opacity.clone(), blending_data);
						let faded_value = f64::from(attribute_value.clone())
							* (f64::from(submaster_opacity) / f64::from(u16::MAX));
						let new_value = f64::from(cumulative_attribute.clone()) + faded_value;
						*cumulative_attribute = new_value.clamp(u16::MIN as f64, u16::MAX as f64).round() as u16;
					}
					BlenderValue::Static(attribute_value) => {
						match blending_data.scheme {
							BlendingScheme::HTP => {
								let submaster_opacity =
									opacity_modifier(opacity.clone(), blending_data);
								let faded_value = f64::from(attribute_value.clone())
									* (f64::from(submaster_opacity) / f64::from(u16::MAX));
								let faded_value = faded_value.clamp(0f64, 65535f64).round() as u16;
								if &faded_value > cumulative_attribute {
									*cumulative_attribute = faded_value;
								}
							}
							BlendingScheme::LTP => {
								// Blend and insert faded value
								*cumulative_attribute = blend_ltp(
									*cumulative_attribute,
									attribute_value.clone(),
									opacity.clone(),
								);
							}
						};
					}
				};
			}
		}
	}
}

pub fn blend_layer(
	cumulative_layer: &mut FullMixerOutput,
	data_sources: &LayerDataSourcesLocked,
	opacity: u16,
	submaster: &SubmasterData,
) {
	let blend_fixture_with_checks = |(fixture_id, cumulative_fixture)| {
		if let Some(submaster_values) = submaster.get(fixture_id) {
			blend_fixture(
				fixture_id,
				cumulative_fixture,
				data_sources,
				opacity,
				submaster_values,
			);
		}
	};
	// Use rayon to split the blending by fixture across multiple threads
	cumulative_layer
		.par_iter_mut()
		.for_each(blend_fixture_with_checks);
	// cumulative_layer
	// 	.iter_mut()
	// 	.for_each(blend_fixture_with_checks);
}

/// Aids in snapping, so an attribute that uses the "snapping" feature doesn't blend.
///
/// If the opacity is past the snapping threshold, this function returns maximum opacity.
/// If it is below the threshold, it returns 0.
fn opacity_modifier(submaster_opacity: u16, blending_data: &BlendingData) -> u16 {
	return match blending_data.snap {
		SnapData::NoSnap => submaster_opacity,
		SnapData::SnapAt(fulcrum) => {
			if submaster_opacity > fulcrum {
				u16::MAX
			} else {
				u16::MIN
			}
		}
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
