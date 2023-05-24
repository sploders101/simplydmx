<script lang="ts" setup>
	import { reactive, ref, computed, onMounted, nextTick, watch, toRaw, PropType } from 'vue';
	import { usePatcherState } from "@/stores/patcher";
	import { ActiveSelection, Canvas, Circle, Object as FabricObject } from "fabric";
	import { useElementBounding } from '@vueuse/core';
	import {
		exhaustiveMatch,
		exhaustiveMatchOriginal,
		patcher,
		ControlGroup,
		FixtureInfo,
		Personality,
		SubmasterData,
		FullMixerOutput,
		FixtureMixerOutput,
	} from '@/scripts/api/ipc';
	import { cmyk2rgb, normalizeChannel } from "@/scripts/conversions";
	import { VisibleControlGroup } from "./types";
	import { createGradient } from "./helpers";
	import FixtureControl from "./controls/FixtureControl.vue";

	let patcherState = usePatcherState();

	let props = defineProps({
		displayData: {
			required: true,
			type: Object as PropType<FullMixerOutput | SubmasterData | null>,
		},
		updateProps: {
			required: true,
			type: Function as PropType<(props: SubmasterData) => Promise<void>>,
		},
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

	// Update light values when displayData changes, but delay 1 tick
	// to give the patcher watcher a chance to update first
	watch(() => [props.displayData, ...fixtures.keys()], () => {
		if (!props.displayData) return;

		// Fixtures should be created/destroyed by patcher state
		for (const fixtureId of fixtures.keys()) {
			if (!props.displayData[fixtureId]) continue;
			updateLight(fixtureId);
		}
		if (vis.value) vis.value.requestRenderAll();
	}, { immediate: true, deep: true });

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
			|| !fixtures.has(fixtureId)
			|| !props.displayData
		) return null;

		let fixtureInstance = patcherState.value.fixtures[fixtureId];
		if (!fixtureInstance) return null;
		let fixtureTypeInfo = patcherState.value.library[fixtureInstance.fixture_id];
		if (!fixtureTypeInfo) return null;
		let fixturePersonalityInfo = fixtureTypeInfo.personalities[fixtureInstance.personality];
		if (!fixturePersonalityInfo) return null;
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

	function getDefaultFixtureValues(fixtureProfile: FixtureInfo, personality: Personality): FixtureMixerOutput {
		let values: FixtureMixerOutput = {};
		for (const channelId of personality.available_channels) {
			values[channelId] = fixtureProfile.channels[channelId].default || 0;
		}
		return values;
	}

	/**
	 * This function updates a light, pulling from registered stores to get the
	 * needed information. If not all information is available, it will perform
	 * an early return. This function should be called individually for all types
	 * of updates so the data has a chance to synchronize.
	 */
	async function updateLight(fixtureId: string) {
		// await new Promise((res) => setTimeout(res, 1000));
		// Collect data and drop unsynchronized triggers
		if (
			!patcherState.value
			|| !fixtures.has(fixtureId)
			|| !props.displayData
		) return;

		let fixtureInstance = patcherState.value.fixtures[fixtureId];
		if (!fixtureInstance) return;
		let fixtureTypeInfo = patcherState.value.library[fixtureInstance.fixture_id];
		if (!fixtureTypeInfo) return;
		let fixturePersonalityInfo = fixtureTypeInfo.personalities[fixtureInstance.personality];
		if (!fixturePersonalityInfo) return;
		let fixtureData = props.displayData[fixtureId] || getDefaultFixtureValues(fixtureTypeInfo, fixturePersonalityInfo);
		let fixtureVis = fixtures.get(fixtureId);
		if (!fixtureVis) return;


		// Get data bounds
		let intensityValue = 0;
		let redValue = 255;
		let greenValue = 255;
		let blueValue = 255;
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
				intensityValue = normalizeChannel(fixtureTypeInfo, fixtureData, intensity, "percentage");
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
	}, { immediate: true });

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
					exhaustiveMatchOriginal(group.channels, {
						Intensity: (controlData) => intensity.controls.push({
							instanceId: fixtureId!,
							controlData,
						}),
						RGBGroup: (controlData) => colorGroup.controls.push({
							instanceId: fixtureId!,
							controlData,
						}),
						CMYKGroup: (controlData) => colorGroup.controls.push({
							instanceId: fixtureId!,
							controlData,
						}),
						ColorWheel: (controlData) => {
							const existingGroup = other[otherKey];
							if (existingGroup) return existingGroup;
							const group: VisibleControlGroup = {
								name: "Color Wheel",
								type: "selections",
								controls: [],
							};
							other[otherKey] = group;
							group.controls.push({
								instanceId: fixtureId!,
								controlData,
							});
						},
						Gobo: (controlData) => {
							const existingGroup = other[otherKey];
							if (existingGroup) return existingGroup;
							const group: VisibleControlGroup = {
								name: "Gobo",
								type: "selections",
								controls: [],
							};
							other[otherKey] = group;
							group.controls.push({
								instanceId: fixtureId!,
								controlData,
							});
						},
						PanTilt: (controlData) => position.controls.push({
							instanceId: fixtureId!,
							controlData,
						}),
						Zoom: (controlData) => zoom.controls.push({
							instanceId: fixtureId!,
							controlData,
						}),
						GenericInput: (controlData) => {
							const existingGroup = other[otherKey];
							if (existingGroup) return existingGroup;
							const group: VisibleControlGroup = {
								name: "Generic Input",
								type: "fader",
								controls: [],
							};
							other[otherKey] = group;
							group.controls.push({
								instanceId: fixtureId!,
								controlData,
							});
						},
					});
				} else {
					// Non-standard control. Add fixture/personality-specific group
					const existingGroup = other[otherKey];
					if (existingGroup) return existingGroup;
					const visibleGroup: VisibleControlGroup = exhaustiveMatchOriginal(group.channels, {
						Intensity: (controlData) => ({
							name: group.name!,
							type: "fader",
							controls: [{ instanceId: fixtureId!, controlData }],
						}) as const,
						CMYKGroup: (controlData) => ({
							name: group.name!,
							type: "color",
							controls: [{ instanceId: fixtureId!, controlData }],
						}) as const,
						RGBGroup: (controlData) => ({
							name: group.name!,
							type: "color",
							controls: [{ instanceId: fixtureId!, controlData }],
						}) as const,
						ColorWheel: (controlData) => ({
							name: group.name!,
							type: "selections",
							controls: [{ instanceId: fixtureId!, controlData }],
						}) as const,
						PanTilt: (controlData) => ({
							name: group.name!,
							type: "position",
							controls: [{ instanceId: fixtureId!, controlData }],
						}) as const,
						Gobo: (controlData) => ({
							name: group.name!,
							type: "selections",
							controls: [{ instanceId: fixtureId!, controlData }],
						}) as const,
						Zoom: (controlData) => ({
							name: group.name!,
							type: "fader",
							controls: [{ instanceId: fixtureId!, controlData }],
						}) as const,
						GenericInput: (controlData) => ({
							name: group.name!,
							type: "fader",
							controls: [{ instanceId: fixtureId!, controlData }],
						}) as const,
					});
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
		vis.value.on("selection:created", (event) => {
			selected.value = event.selected;
			const selectionObj = vis.value?.getActiveSelection();
			if (selectionObj instanceof ActiveSelection) {
				selectionObj.hasControls = false;
			}
		});
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
					Math.round(coords.x),
					Math.round(coords.y),
				);
			});
		});

		fixtures.forEach((fixtureObj) => vis.value!.add(fixtureObj));
	}));

	let pushInProgress = false;
	let batchedData: SubmasterData | null = null;

	function handleBatchedProps() {
		if (batchedData === null) {
			pushInProgress = false;
		} else {
			const myBatchedData = batchedData;
			batchedData = null;
			props.updateProps(myBatchedData)
				.then(() => handleBatchedProps());
		}
	}

	function handleUpdatedProps(data: SubmasterData) {
		if (pushInProgress) {
			// Merge data
			if (batchedData === null) {
				batchedData = data;
			} else {
				for (const [fixtureId, values] of Object.entries(data)) {
					const fixtureData = batchedData[fixtureId];
					if (fixtureData) {
						// Merge data
						for (const [channelId, value] of Object.entries(values)) {
							fixtureData[channelId] = value;
						}
					} else {
						batchedData[fixtureId] = values;
					}
				}
			}
		} else {
			pushInProgress = true;
			props.updateProps(data)
				.then(() => handleBatchedProps());
		}
	}

</script>

<template>
	<div class="sdmx-visualizer">
		<div ref="viewport" class="sdmx-visualizer__viewport">
			<canvas
				ref="canvas"
				/>
		</div>
		<div class="sdmx-visualizer__control-panel" v-if="props.displayData">
			<FixtureControl
				:group="group"
				:display-data="props.displayData"
				@update-props="handleUpdatedProps($event)"
				v-for="group in activeControlGroups"
				/>
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

			gap: 0.5rem;
			padding: 0.5rem;
		}
	}
</style>
