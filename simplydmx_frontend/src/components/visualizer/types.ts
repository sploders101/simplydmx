import { UnionToIntersection } from "@vue/shared";
import { ControlGroupData } from "@/scripts/api/ipc/rpc";

export type ControlGroupTypes = keyof UnionToIntersection<ControlGroupData>;
export type ControlGroupByName<Name extends ControlGroupTypes> = Name extends string
	? { [K in Name]: UnionToIntersection<ControlGroupData>[K] }
	: never;

export type VisibleCGTypes =
	| "fader"
	| "color"
	| "position"
	| "selections"

export interface TypeFromCGData extends Record<string, VisibleCGTypes> {
	RGBGroup: "color",
	CMYKGroup: "color",
	PanTilt: "position",
	Gobo: "selections",
	ColorWheel: "color",
	Zoom: "fader",
	GenericInput: "fader",
}

export interface VisibleControlTypes extends Record<VisibleCGTypes, ControlGroupData> {
	color: ControlGroupByName<"RGBGroup" | "CMYKGroup">,
	position: ControlGroupByName<"PanTilt">,
	fader: ControlGroupByName<"Intensity" | "Zoom" | "GenericInput">,
	selections: ControlGroupByName<"Gobo" | "ColorWheel">,
}

export interface VisibleControlGroup<ControlType extends VisibleCGTypes = VisibleCGTypes> {
	name: string,
	type: ControlType,
	controls: Array<{
		instanceId: string,
		controlData: VisibleControlTypes[ControlType],
	}>,
}
