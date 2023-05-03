<script lang="ts" setup>
	import { reactive, ref, computed, onMounted, nextTick, watch, toRaw } from 'vue';
	import { usePatcherState } from "@/stores/patcher";
	import { useLiveMixState } from "@/stores/live";
	import { ActiveSelection, Canvas, Circle, Gradient, Object as FabricObject } from "fabric";
	import { useElementBounding } from '@vueuse/core';
	import { type ControlGroup, exhaustiveMatch, patcher, FixtureInfo } from '@/scripts/api/ipc';
	import { cmyk2rgb, normalizeChannel } from "@/scripts/conversions";
	import { VisibleControlGroup } from "./types";

	let patcherState = usePatcherState();
	let liveMix = useLiveMixState();
	let displayData = computed(() => {
		return liveMix.value;
	});

	/** The canvas that fabric.js should render into */
	const canvas = ref<HTMLCanvasElement | null>(null);

	/** The fabric.js canvas controller */
	const vis = ref<Canvas | null>(null);

	/** The viewport element (the container the canvas should fill) */
	const viewport = ref<HTMLDivElement | null>(null);

	/** The reactive boundaries of the viewport (ie. the area the canvas should fill) */
	const viewportBounds = useElementBounding(viewport);

	/** Map of fixture IDs to fabric objects */
	const fixtures = reactive(new Map<string, Circle>());

	/**
	 * Map of fabric objects to fixture IDs.
	 * Weakly typed due to the number of incompatible fabric object types
	 */
	const fixtureObjToId = new WeakMap<any, string>();

	watch(displayData, () => {
		if (!displayData.value) return;

		// Fixtures should be created/destroyed by patcher state
		for (const fixtureId of fixtures.keys()) {
			if (!displayData.value[fixtureId]) continue;
			updateLight(fixtureId);
		}
		if (vis.value) vis.value.requestRenderAll();
	}, { immediate: true });

	interface FixtureProfileIds {
		profileId: string,
		profile: FixtureInfo,
		personalityId: string,
	}

	function getFixtureProfileIds(fixtureId: string): FixtureProfileIds | null {
		if (!patcherState.value) return null;

		let fixtureInstance = patcherState.value.fixtures[fixtureId];
		if (!fixtureInstance) return null;
		let fixtureTypeInfo = patcherState.value.library[fixtureInstance.fixture_id];
		if (!fixtureTypeInfo) return null;

		return {
			profileId: fixtureInstance.fixture_id,
			profile: fixtureTypeInfo,
			personalityId: fixtureInstance.personality,
		};
	}

	/**
	 * Gets the active control groups of a given fixture based on its selected personality.
	 */
	function getActiveControlGroups(fixtureId: string): ControlGroup[] | null {
		// Collect data and drop unsynchronized triggers
		if (
			!patcherState.value
			|| !vis.value
			|| !fixtures.has(fixtureId)
			|| !displayData.value
		) return null;

		let fixtureInstance = patcherState.value.fixtures[fixtureId];
		if (!fixtureInstance) return null;
		let fixtureTypeInfo = patcherState.value.library[fixtureInstance.fixture_id];
		if (!fixtureTypeInfo) return null;
		let fixturePersonalityInfo = fixtureTypeInfo.personalities[fixtureInstance.personality];
		if (!fixturePersonalityInfo) return null;
		let fixtureData = displayData.value[fixtureId];
		if (!fixtureData) return null;
		let fixtureVis = fixtures.get(fixtureId);
		if (!fixtureVis) return null;
		let availableChannels = fixturePersonalityInfo.available_channels;

		const activeControlGroups = fixtureTypeInfo.control_groups.filter((group) => exhaustiveMatch(group.channels, {
				"RGBGroup": ({ red, green, blue }) => includesAll(
					availableChannels,
					[red, green, blue],
				),
				"CMYKGroup": ({ cyan, magenta, yellow, black }) => includesAll(
					availableChannels,
					[cyan, magenta, yellow, black],
				),
				"ColorWheel": (wheel) => availableChannels.includes(wheel),
				"GenericInput": (channel) => availableChannels.includes(channel),
				"Gobo": (gobo) => availableChannels.includes(gobo),
				"Intensity": (intensity) => availableChannels.includes(intensity),
				"PanTilt": ({ pan, tilt }) => includesAll(
					availableChannels,
					[pan, tilt],
				),
				"Zoom": (zoom) => availableChannels.includes(zoom),
		}));
		return activeControlGroups;
	}

	/** Returns `true` if all items from `criteria` were found in `source` */
	function includesAll<T>(source: T[], criteria: T[]) {
		return !criteria.some((item) => !source.includes(item));
	}

	/**
	 * This function updates a light, pulling from registered stores to get the
	 * needed information. If not all information is available, it will perform
	 * an early return. This function should be called individually for all types
	 * of updates so the data has a chance to synchronize.
	 */
	function updateLight(fixtureId: string) {
		// Collect data and drop unsynchronized triggers
		if (
			!patcherState.value
			|| !vis.value
			|| !fixtures.has(fixtureId)
			|| !displayData.value
		) return;

		let fixtureInstance = patcherState.value.fixtures[fixtureId];
		if (!fixtureInstance) return;
		let fixtureTypeInfo = patcherState.value.library[fixtureInstance.fixture_id];
		if (!fixtureTypeInfo) return;
		let fixturePersonalityInfo = fixtureTypeInfo.personalities[fixtureInstance.personality];
		if (!fixturePersonalityInfo) return;
		let fixtureData = displayData.value[fixtureId];
		if (!fixtureData) return;
		let fixtureVis = fixtures.get(fixtureId);
		if (!fixtureVis) return;

		// Get data bounds
		let intensityValue = 0;
		let redValue = 0;
		let greenValue = 0;
		let blueValue = 0;
		let controlGroups = getActiveControlGroups(fixtureId);

		if (controlGroups) controlGroups.forEach((group) => exhaustiveMatch(group.channels, {
			CMYKGroup: ({ cyan, magenta, yellow, black }) => {
				const normalCyan = normalizeChannel(fixtureTypeInfo, fixtureData, cyan);
				const normalMagenta = normalizeChannel(fixtureTypeInfo, fixtureData, magenta);
				const normalYellow = normalizeChannel(fixtureTypeInfo, fixtureData, yellow);
				const normalBlack = normalizeChannel(fixtureTypeInfo, fixtureData, black);
				let rgb = cmyk2rgb(normalCyan, normalMagenta, normalYellow, normalBlack);
				redValue = rgb.red;
				greenValue = rgb.green;
				blueValue = rgb.blue;
			},
			ColorWheel: (color) => {
				// TODO: Needs mapped to fixture profile
			},
			GenericInput: (_channel) => {},
			Gobo: (_gobo) => {},
			Intensity: (intensity) => {
				intensityValue = normalizeChannel(fixtureTypeInfo, fixtureData, intensity);
			},
			PanTilt: ({ pan, tilt }) => {},
			RGBGroup: ({ red, green, blue }) => {
				redValue = normalizeChannel(fixtureTypeInfo, fixtureData, red);
				greenValue = normalizeChannel(fixtureTypeInfo, fixtureData, green);
				blueValue = normalizeChannel(fixtureTypeInfo, fixtureData, blue);
			},
			Zoom: (zoom) => {},
		}));
		
		// Update the light
		fixtureVis.set("fill", createGradient(intensityValue, redValue, greenValue, blueValue));
	}

	// Keep fabric dimensions up-to-date with the container
	watch([viewportBounds.height, viewportBounds.width], () => {
		if (vis.value) {
			vis.value.setDimensions({
				width: viewportBounds.width.value,
				height: viewportBounds.height.value,
			}, {});
		}
	});

	// Update the circles when the patch changes
	watch(() => patcherState.value?.fixtures, () => {
		// If we're missing the patcher state, we shouldn't be displaying anything,
		// so clear the canvas of all fixtures
		if (!patcherState.value) {
			if (vis.value) fixtures.forEach((fixtureObj) => vis.value!.remove(fixtureObj));
			fixtures.clear();
			return;
		}

		// We have patcher data, so diff the fixtures
		let foundFixtures: string[] = [];
		Object.values(patcherState.value.fixtures).forEach((fixture) => {
			if (!fixtures.has(fixture.id)) {
				let fixtureObj = new Circle({
					stroke: "#FFFFFF",
					radius: 15,
					strokeWidth: 1,
					fill: createGradient(0, 0, 0, 0),
					left: fixture.visualization_info.x,
					top: fixture.visualization_info.y,
					hasControls: false,
				});
				fixtures.set(fixture.id, fixtureObj);
				fixtureObjToId.set(fixtureObj, fixture.id);
				foundFixtures.push(fixture.id);
				updateLight(fixture.id);
				if (vis.value) vis.value.add(fixtureObj);
			}
		});
		let toRemove: string[] = [];
		fixtures.forEach((_, fixture_id) => {
			if (!foundFixtures.includes(fixture_id)) {
				toRemove.push(fixture_id);
			}
		});
		toRemove.forEach((fixture_id) => {
			if (vis.value) vis.value.remove(fixtures.get(fixture_id)!);
			fixtures.delete(fixture_id);
		});
		if (vis.value) vis.value.requestRenderAll();
	}, { immediate: true });

	/**
	 * Creates a fill gradient for a light's visualization
	 *
	 * Intensity is 0-1 floating-point.
	 *
	 * Red, green, and blue are 0-255.
	 */
	function createGradient(intensity: number, red: number, green: number, blue: number) {
		const color = `rgb(${red}, ${green}, ${blue})`;
		return new Gradient({
			type: "radial",
			gradientUnits: "pixels",
			gradientTransform: [15, 0, 0, 15, 15, 15],
			coords: { r1: 0, r2: 1, x1: 0, x2: 0, y1: 0, y2: 0 },
			colorStops: [
				{ offset: 0, color, opacity: 1 } as any,
				{ offset: 0.5, color, opacity: 1 * intensity } as any,
				{ offset: 0.9, color, opacity: 0 } as any,
			],
		});
	}

	const selected = ref<FabricObject[]>([]);

	const activeControlGroups = computed(() => {
		const intensity: VisibleControlGroup<"fader"> = {
			name: "Intensity",
			type: "fader",
			controls: [],
		};
		const colorGroup: VisibleControlGroup<"color"> = {
			name: "Color",
			type: "color",
			controls: [],
		};
		const position: VisibleControlGroup<"position"> = {
			name: "Position",
			type: "position",
			controls: [],
		};
		const zoom: VisibleControlGroup<"fader"> = {
			name: "Zoom",
			type: "fader",
			controls: [],
		};
		/** Map of fixtureProfileId-cgIndex to `VisibleControlGroup` */
		const other: Record<string, VisibleControlGroup> = {};

		selected.value.forEach((fixtureObj) => {
			let fixtureId = fixtureObjToId.get(toRaw(fixtureObj));
			if (!fixtureId) return;
			let profile = getFixtureProfileIds(fixtureId);
			if (!profile) return;
			let controlGroups = getActiveControlGroups(fixtureId);
			if (!controlGroups) return;

			controlGroups.forEach((group) => {
				const cgIndex = profile!.profile.control_groups.indexOf(group);
				const otherKey = `${profile!.profileId}-${cgIndex}`;
				if (group.name === null) {
					// Standard control. Add multi-output group.
					exhaustiveMatch(group.channels, {
						Intensity: () => intensity,
						RGBGroup: () => colorGroup,
						CMYKGroup: () => colorGroup,
						ColorWheel: () => {
							const existingGroup = other[otherKey];
							if (existingGroup) return existingGroup;
							const group: VisibleControlGroup = {
								name: "Color Wheel",
								type: "selections",
								controls: [],
							};
							other[otherKey] = group;
							return group;
						},
						Gobo: () => {
							const existingGroup = other[otherKey];
							if (existingGroup) return existingGroup;
							const group: VisibleControlGroup = {
								name: "Gobo",
								type: "selections",
								controls: [],
							};
							other[otherKey] = group;
							return group;
						},
						PanTilt: () => position,
						Zoom: () => zoom,
						GenericInput: () => {
							const existingGroup = other[otherKey];
							if (existingGroup) return existingGroup;
							const group: VisibleControlGroup = {
								name: "Generic Input",
								type: "fader",
								controls: [],
							};
							other[otherKey] = group;
							return group;
						},
					}).controls.push({
						instanceId: fixtureId!,
						controlData: group.channels,
					});
				} else {
					// Non-standard control. Add fixture/personality-specific group
					const existingGroup = other[otherKey];
					if (existingGroup) return existingGroup;
					const visibleGroup: VisibleControlGroup = {
						name: group.name!,
						type: exhaustiveMatch(group.channels, {
							Intensity: () => "fader" as const,
							CMYKGroup: () => "color" as const,
							RGBGroup: () => "color" as const,
							ColorWheel: () => "selections" as const,
							PanTilt: () => "position" as const,
							Gobo: () => "selections" as const,
							Zoom: () => "fader" as const,
							GenericInput: () => "fader" as const,
						}),
						controls: [{
							instanceId: fixtureId!,
							controlData: group.channels,
						}],
					};
					other[otherKey] = visibleGroup;
					return visibleGroup;
				}
			});
		});

		// Add known groups
		const groups: VisibleControlGroup[] = [];
		[
			intensity,
			colorGroup,
			position,
			zoom,
		].forEach((group) => group.controls.length && groups.push(group));

		// Add unknown groups
		Object.keys(other).sort().forEach((key) => groups.push(other[key]));

		return groups;
	});

	onMounted(() => nextTick(() => {
		if (!canvas.value) return console.log("Canvas missing in visualizer");

		vis.value = new Canvas(canvas.value, {
			enableRetinaScaling: true,
			width: viewportBounds.width.value,
			height: viewportBounds.height.value,
			uniformScaling: true,
		});

		// Keep selection object updated so we can update the control panel
		vis.value.on("selection:created", (event) => selected.value = event.selected);
		vis.value.on("selection:updated", (event) => {
			event.selected.forEach((obj) => selected.value.push(obj));
			if (event.deselected.length) selected.value = selected.value.filter((obj) => {
				return !event.deselected.includes(toRaw(obj) as any);
			});
		});
		vis.value.on("selection:cleared", () => selected.value = []);

		vis.value.on("object:modified", (event) => {
			let objects = event.target instanceof ActiveSelection
				? event.target.getObjects()
				: [event.target];
			objects.forEach((fixtureObject) => {
				let fixtureId = fixtureObjToId.get(fixtureObject);
				let coords = fixtureObject.getCoords(true)[0];
				if (fixtureId) patcher.edit_fixture_placement(
					fixtureId,
					coords.x,
					coords.y,
				);
			});
		});

		fixtures.forEach((fixtureObj) => vis.value!.add(fixtureObj));
	}));

</script>

<template>
	<div class="sdmx-visualizer">
		<div ref="viewport" class="sdmx-visualizer__viewport">
			<canvas
				ref="canvas"
				/>
		</div>
		<div class="sdmx-visualizer__control-panel">
			<div class="test-control" v-for="group in activeControlGroups">
				{{ group.name }}
			</div>
		</div>
	</div>
</template>

<style lang="scss">
	.sdmx-visualizer {
		// background-color: red;
		width: 100%;
		height: 100%;

		overflow: auto;
		display: flex;
		flex-flow: column nowrap;

		.sdmx-visualizer__viewport {
			flex: 1 1 0;
			position: relative;
			height: 100%;
			overflow: auto;
		}

		.sdmx-visualizer__control-panel {
			height: 15rem;
			background-color: var(--visualizer-control-panel-background);

			display: flex;
			flex-flow: row nowrap;
			overflow: auto;
			align-items: stretch;

			gap: 1rem;

			.test-control {
				background-color: red;
				border: 2px solid black;
				min-width: 5rem;

				text-align: center;
				display: flex;
				flex-flow: column nowrap;
				justify-content: center;
			}
		}
	}
</style>
