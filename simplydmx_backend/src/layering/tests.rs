use crate::patcher::fixture_types::{
	BlendingScheme, Channel, ChannelSize, ChannelType, ChannelValue, Segment, SegmentDisplay, SnapData,
};

use super::blend;

#[test]
fn test_blend_ltp() {
	let channel = Channel {
		name: "Test Channel".into(),
		intensity_emulation: None,
		size: ChannelSize::U16,
		default: ChannelValue::L16(0),
		ch_type: ChannelType::Linear {
			priority: BlendingScheme::LTP,
		},
	};
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(91),
			ChannelValue::L16(152),
			13600
		),
		ChannelValue::L16(103)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(5000),
			ChannelValue::L16(11000),
			49000
		),
		ChannelValue::L16(9486)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(43400),
			ChannelValue::L16(11000),
			59000
		),
		ChannelValue::L16(14231)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(20500),
			ChannelValue::L16(0),
			65535
		),
		ChannelValue::L16(0)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(20500),
			ChannelValue::L16(65535),
			65535
		),
		ChannelValue::L16(65535)
	);
}

#[test]
fn test_blend_htp() {
	let channel = Channel {
		name: "Test Channel".into(),
		intensity_emulation: None,
		size: ChannelSize::U16,
		default: ChannelValue::L16(0),
		ch_type: ChannelType::Linear {
			priority: BlendingScheme::HTP,
		},
	};
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(91),
			ChannelValue::L16(152),
			13600
		),
		ChannelValue::L16(91)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(5000),
			ChannelValue::L16(11000),
			49000
		),
		ChannelValue::L16(8224)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(43400),
			ChannelValue::L16(11000),
			59000
		),
		ChannelValue::L16(43400)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(20500),
			ChannelValue::L16(0),
			65535
		),
		ChannelValue::L16(20500)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(20500),
			ChannelValue::L16(65535),
			65535
		),
		ChannelValue::L16(65535)
	);
}

#[test]
fn test_blend_snap_ltp() {
	let channel = Channel {
		name: "Test Channel".into(),
		intensity_emulation: None,
		size: ChannelSize::U16,
		default: ChannelValue::L16(0),
		ch_type: ChannelType::Segmented {
			priority: BlendingScheme::LTP,
			snapping: SnapData::SnapAt(ChannelValue::L16(22000)),
			segments: vec![
				Segment {
					start: ChannelValue::L16(0),
					end: ChannelValue::L16(255),
					name: "Red".into(),
					display: SegmentDisplay::Color {
						red: 255,
						green: 0,
						blue: 0,
					},
				},
				Segment {
					start: ChannelValue::L16(256),
					end: ChannelValue::L16(511),
					name: "Green".into(),
					display: SegmentDisplay::Color {
						red: 0,
						green: 255,
						blue: 0,
					},
				},
				Segment {
					start: ChannelValue::L16(512),
					end: ChannelValue::L16(767),
					name: "Blue".into(),
					display: SegmentDisplay::Color {
						red: 0,
						green: 0,
						blue: 255,
					},
				},
			]
		},
	};
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(91),
			ChannelValue::L16(152),
			13600
		),
		ChannelValue::L16(91)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(5000),
			ChannelValue::L16(11000),
			49000
		),
		ChannelValue::L16(11000)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(43400),
			ChannelValue::L16(11000),
			59000
		),
		ChannelValue::L16(11000)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(20500),
			ChannelValue::L16(0),
			65535
		),
		ChannelValue::L16(0)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(20500),
			ChannelValue::L16(65535),
			65535
		),
		ChannelValue::L16(65535)
	);
}

#[test]
fn test_blend_snap_htp() {
	let channel = Channel {
		name: "Test Channel".into(),
		intensity_emulation: None,
		size: ChannelSize::U16,
		default: ChannelValue::L16(0),
		ch_type: ChannelType::Segmented {
			priority: BlendingScheme::HTP,
			snapping: SnapData::SnapAt(ChannelValue::L16(22000)),
			segments: vec![
				Segment {
					start: ChannelValue::L16(0),
					end: ChannelValue::L16(255),
					name: "Red".into(),
					display: SegmentDisplay::Color {
						red: 255,
						green: 0,
						blue: 0,
					},
				},
				Segment {
					start: ChannelValue::L16(256),
					end: ChannelValue::L16(511),
					name: "Green".into(),
					display: SegmentDisplay::Color {
						red: 0,
						green: 255,
						blue: 0,
					},
				},
				Segment {
					start: ChannelValue::L16(512),
					end: ChannelValue::L16(767),
					name: "Blue".into(),
					display: SegmentDisplay::Color {
						red: 0,
						green: 0,
						blue: 255,
					},
				},
			]
		},
	};
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(91),
			ChannelValue::L16(152),
			13600
		),
		ChannelValue::L16(91)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(5000),
			ChannelValue::L16(11000),
			49000
		),
		ChannelValue::L16(11000)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(43400),
			ChannelValue::L16(11000),
			59000
		),
		ChannelValue::L16(43400)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(20500),
			ChannelValue::L16(0),
			65535
		),
		ChannelValue::L16(20500)
	);
	assert_eq!(
		blend(
			&channel,
			ChannelValue::L16(20500),
			ChannelValue::L16(65535),
			65535
		),
		ChannelValue::L16(65535)
	);
}
