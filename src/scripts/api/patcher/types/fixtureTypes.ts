import { BlendingScheme, SnapData } from "../../types/mixer";
import { DMXFixtureData } from "../../outputDrivers/dmx/types";

export type Uuid = string;

export type FixtureBundle =
	| GenericFixtureBundle<"DMX", DMXFixtureData>
;

export interface GenericFixtureBundle<C extends string, T> {
	fixture_info: FixtureInfo<C>,
	output_info: T;
}

export interface FixtureInfo<C extends string = string> {
	id: Uuid,
	name: string,
	short_name?: string | null,
	manufacturer?: string | null,
	family?: string | null,
	metadata: FixtureMeta,
	channels: Record<string, Channel>,
	personalities: Record<string, Personality>,
	output_driver: C,
}

export interface FixtureMeta {
	manufacturer?: string | null,
	manual_link?: string | null,
}

export interface Channel {
	size: ChannelSize,
	default?: number | null,
	ch_type: ChannelType,
}

export type ChannelSize = "U8" | "U16";

export type ChannelType = SegmentedChannel | LinearChannel;

export interface SegmentedChannel {
	type: "Segmented",
	segments: Segment[],
	priority: BlendingScheme,
	snapping?: SnapData | null,
}

export interface LinearChannel {
	type: "Linear",
	priority: BlendingScheme,
}

export interface Segment {
	start: number,
	end: number,
	name: string,
	id: string,
}

export interface Personality {
	available_channels: string[];
}

/** Identifies an individual instance of a fixture */
export interface  FixtureInstance {

	/** The ID of this particular fixture */
	id: Uuid,

	/** The ID of this fixture's type */
	fixture_id: Uuid,

	/** The personality identifier of this fixture */
	personality: string,

	/** An arbitrary name for this particular instance of the fixture */
	name: string | null,

	/** Arbitrary comments about this particular instance left by the user */
	comments: string | null,

}

export interface PatcherState {
	library: Record<Uuid, FixtureInfo>,
	fixture_order: Uuid[],
	fixtures: Record<Uuid, FixtureInstance>,
}
