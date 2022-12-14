import { callService } from "./agnostic_abstractions";


/**
 * Represents the abstract data for a single light in a layer.
 * A value's binary may be masked if the output is u8 (integer overflow cast)
 */
export type AbstractLayerLight = Record<string, BlenderValue>;

/**
 * Value to be used in a submaster with instructions for mixing it into the result
 */
export type BlenderValue = { type: "None" } | { type: "Static"; value: number } | { type: "Offset"; value: number };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface BlendingData {
    scheme: BlendingScheme;
    snap: SnapData;
    allow_wrap: boolean;
    max_value: number;
    min_value: number;
}

/**
 * The method in which conflicts are resolved while blending
 */
export type BlendingScheme = "HTP" | "LTP";

/**
 * Information about a specific channel available on the fixture
 */
export interface Channel {
    size: ChannelSize;
    default?: number;
    ch_type: ChannelType;
}

/**
 * Dictates the size of the output. Values will be stored as the largest of these options, but bounds
 * will be enforced by the UI, mixer, and output will be truncated.
 */
export type ChannelSize = "U8" | "U16";

/**
 * Describes information used for controlling and blending the channel
 */
export type ChannelType = { type: "Segmented"; segments: Segment[]; priority: BlendingScheme; snapping: SnapData | null } | { type: "Linear"; priority: BlendingScheme };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type CreateFixtureError = "FixtureTypeMissing" | "ControllerMissing" | { ErrorFromController: CreateInstanceError };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type CreateInstanceError = { type: "InvalidData" } | ({ type: "Other" } & string) | { type: "Unknown" };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface DMXFixtureData {
    personalities: Record<string, DMXPersonalityData>;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface DMXFixtureInstance {
    universe: Uuid | null;
    offset: number | null;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type DMXInitializationError = "UnrecognizedData";

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface DMXPersonalityData {
    dmx_channel_order: string[];
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface DMXShowSave {
    library: Record<Uuid, DMXFixtureData>;
    fixtures: Record<Uuid, DMXFixtureInstance>;
    universes: Record<Uuid, UniverseInstance>;
}

/**
 * Minified representation of a DMX driver for display
 */
export interface DisplayableDMXDriver {
    id: string;
    name: string;
    description: string;
}

/** Describes a value to be shown in a dropdown list */
export interface DropdownOptionJSON {
    name: string;
    description: string | null;
    value: Value;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface E131DMXShowSave {
    universes: Record<Uuid, E131Universe>;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type E131InitializationError = "UnrecognizedData";

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface E131Universe {
    external_universe: number;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type EditError = { type: "InvalidData" } | ({ type: "Other" } & string) | { type: "Unknown" };

/** Represents criteria used to filter an event. For example, a submaster UUID could be used to filter submaster updates by that specific submaster */
export type FilterCriteria = { type: "None" } | { type: "String"; data: string } | { type: "Uuid"; data: Uuid };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface FixtureBundle {
    fixture_info: FixtureInfo;
    output_info: SerializedData;
}

/**
 * Data type that contains generic, protocol-erased information about a fixture such as name,
 * metadata, personalities, and references to services within the output controller.
 */
export interface FixtureInfo {
    id: Uuid;
    name: string;
    short_name: string | null;
    manufacturer: string | null;
    family: string | null;
    metadata: FixtureMeta;
    channels: Record<string, Channel>;
    personalities: Record<string, Personality>;
    output_driver: string;
}

/**
 * Identifies an individual instance of a fixture
 */
export interface FixtureInstance {
    id: Uuid;
    fixture_id: Uuid;
    personality: string;
    name: string | null;
    comments: string | null;
}

/**
 * Metadata about the fixture, used for display in the UI
 */
export interface FixtureMeta {
    manufacturer: string | null;
    manual_link: string | null;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type FormDescriptor = FormItem[];

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface FormDropdown {
    label: string;
    id: string;
    item_source: FormItemOptionSource;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type FormItem = { Textbox: FormTextbox } | { Number: FormNumber } | { Dropdown: FormDropdown } | { Section: FormSection } | { VerticalStack: FormItem[] } | { HorizontalStack: FormItem[] };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type FormItemOptionSource = { Static: { values: DropdownOptionJSON[] } } | { TypeSpec: { typespec_id: string } };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface FormNumber {
    label: string;
    id: string;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface FormSection {
    label: string;
    form_items: FormItem[];
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface FormTextbox {
    label: string;
    id: string;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type ImportError = { type: "InvalidData" } | ({ type: "Other" } & string) | { type: "Unknown" };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type ImportFixtureError = "UnknownController" | { ErrorFromController: ImportError };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type JSONCallServiceError = { type: "ServiceNotFound" } | { type: "ArgDeserializationFailed" } | { type: "ResponseSerializationFailed" };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type JSONCommand = { type: "CallService"; message_id: number; plugin_id: string; service_id: string; args: Value[] } | { type: "GetServices"; message_id: number } | { type: "GetOptions"; message_id: number; provider_id: string } | { type: "SendEvent"; name: string; criteria: FilterCriteria | null; data: Value } | { type: "Subscribe"; name: string; criteria: FilterCriteria | null } | { type: "Unsubscribe"; name: string; criteria: FilterCriteria | null };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type JSONResponse = { type: "CallServiceResponse"; message_id: number; result: Value } | { type: "ServiceList"; message_id: number; list: ServiceDescription[] } | { type: "OptionsList"; message_id: number; list: { Ok: DropdownOptionJSON[] } | { Err: TypeSpecifierRetrievalError } } | { type: "CallServiceError"; message_id: number; error: JSONCallServiceError } | { type: "Event"; name: string; criteria: FilterCriteria; data: Value };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type LinkUniverseError = { type: "ErrorFromController"; data: RegisterUniverseError } | { type: "UniverseNotFound" } | { type: "ControllerNotFound" };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface MixerContext {
    default_context: MixingContext;
    frozen_context: MixingContext | null;
    blind_opacity: number;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type MixerInitializationError = "UnrecognizedData";

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface MixingContext {
    layer_order: Uuid[];
    layer_opacities: Record<Uuid, number>;
    user_submasters: Record<Uuid, StaticLayer>;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type PatcherInitializationError = "UnrecognizedData";

/**
 * Identifies non-implementation-specific features of a personality.
 * 
 * Implementation-specific features of a personality such as channel order should
 * should be stored in the output data for use by the output plugin.
 */
export interface Personality {
    available_channels: string[];
}

/**
 * An error returned when registering a saver. This is usually okay to unwrap, since it should be during init
 */
export type RegisterSavableError = { type: "SaverAlreadyExists" };

/**
 * An error that occurs while registering a universe
 */
export type RegisterUniverseError = "InvalidData" | { Other: string } | "Unknown";

/**
 * An error returned by the saver if saving data failed
 */
export type SaveError = { type: "SaverReturnedErr"; data: { error: string } } | { type: "ErrorSerializing"; data: { error: string } } | { type: "Unsafe" };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type SaverInitializationStatus = { type: "FinishedSafe" } | { type: "FinishedUnsafe" } | { type: "Initializing" };

/**
 * Identifies a segment used in a segmented channel
 */
export interface Segment {
    start: number;
    end: number;
    name: string;
    id: string;
}

/**
 * Data type used to hold a serialized instance of an arbitrary data type.
 * 
 * This is intended to encapsulate dynamically-typed data intended for deserialization by the output plugin
 */
export type SerializedData = number[] | Value;

/** Describes an argument that must be passed to a service call */
export interface ServiceArgumentOwned {
    id: string;
    name: string;
    description: string;
    val_type: string;
    val_type_hint: string | null;
}

/** Describes a service that can be called from an external API */
export interface ServiceDescription {
    plugin_id: string;
    id: string;
    name: string;
    description: string;
    arguments: ServiceArgumentOwned[];
    returns: ServiceArgumentOwned | null;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface SharablePatcherState {
    library: Record<Uuid, FixtureInfo>;
    fixture_order: Uuid[];
    fixtures: Record<Uuid, FixtureInstance>;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface ShowFile {
    plugin_data: Record<string, number[]>;
}

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export type SnapData = { type: "NoSnap" } | { type: "SnapAt"; data: number };

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface StaticLayer {
    values: SubmasterData;
}

/**
 * Represents the data within a submaster used for blending
 */
export type SubmasterData = Record<Uuid, AbstractLayerLight>;

/** Describes an error that occurred while retrieving items for a dropdown list */
export type TypeSpecifierRetrievalError = "SpecifierNotFound";

/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */
export interface UniverseInstance {
    id: Uuid;
    name: string;
    controller: string | null;
}

/** Unique identifier used in various parts of the API. In TS, UUID does not have its own data type, so this just re-exports string. */
export type Uuid = string;

/** Represents Rust's `serde_json::Value` type. This is used for dynamic typing, like when using backend-defined forms. */
export type Value = any;


export const core = {
	log(msg: string): Promise<void> { return callService("core", "log", [msg]) },
	log_error(msg: string): Promise<void> { return callService("core", "log_error", [msg]) },
};

export const mixer = {
	commit_blind(): Promise<void> { return callService("mixer", "commit_blind", []) },
	create_layer(): Promise<Uuid> { return callService("mixer", "create_layer", []) },
	delete_layer(submaster_id: Uuid): Promise<boolean> { return callService("mixer", "delete_layer", [submaster_id]) },
	enter_blind_mode(): Promise<void> { return callService("mixer", "enter_blind_mode", []) },
	get_blind_opacity(): Promise<number | null> { return callService("mixer", "get_blind_opacity", []) },
	get_layer_contents(submaster_id: Uuid): Promise<StaticLayer | null> { return callService("mixer", "get_layer_contents", [submaster_id]) },
	get_layer_opacity(submaster_id: Uuid): Promise<number | null> { return callService("mixer", "get_layer_opacity", [submaster_id]) },
	revert_blind(): Promise<void> { return callService("mixer", "revert_blind", []) },
	set_blind_opacity(opacity: number): Promise<void> { return callService("mixer", "set_blind_opacity", [opacity]) },
	set_layer_contents(submaster_id: Uuid, submaster_delta: SubmasterData): Promise<boolean> { return callService("mixer", "set_layer_contents", [submaster_id, submaster_delta]) },
	set_layer_opacity(submaster_id: Uuid, opacity: number, auto_insert: boolean): Promise<boolean> { return callService("mixer", "set_layer_opacity", [submaster_id, opacity, auto_insert]) },
};

export const output_dmx = {
	create_universe(name: string): Promise<Uuid> { return callService("output_dmx", "create_universe", [name]) },
	delete_universe(universe_id: Uuid): Promise<void> { return callService("output_dmx", "delete_universe", [universe_id]) },
	link_universe(universe_id: Uuid, driver: string, form_data: SerializedData): Promise<{ Ok: null } | { Err: LinkUniverseError }> { return callService("output_dmx", "link_universe", [universe_id, driver, form_data]) },
	unlink_universe(universe_id: Uuid): Promise<void> { return callService("output_dmx", "unlink_universe", [universe_id]) },
};

export const patcher = {
	create_fixture(fixture_type: Uuid, personality: string, name: string | null, comments: string | null, form_data: SerializedData): Promise<{ Ok: Uuid } | { Err: CreateFixtureError }> { return callService("patcher", "create_fixture", [fixture_type, personality, name, comments, form_data]) },
	get_patcher_state(): Promise<SharablePatcherState> { return callService("patcher", "get_patcher_state", []) },
	import_fixture(fixture_bundle: FixtureBundle): Promise<{ Ok: null } | { Err: ImportFixtureError }> { return callService("patcher", "import_fixture", [fixture_bundle]) },
};

export const saver = {
	save(): Promise<{ Ok: number[] } | { Err: SaveError }> { return callService("saver", "save", []) },
};

