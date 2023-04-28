<script lang="ts" setup>
	import { computed, reactive, ref, onMounted, nextTick, watch } from 'vue';
	import { usePatcherState } from "@/stores/patcher";
	import { useLiveMixState } from "@/stores/live";
	import { ActiveSelection, Canvas, Circle, Gradient, Object as FabricObject } from "fabric";
	import { useElementBounding } from '@vueuse/core';
	import { ControlGroup, exhaustiveMatch, FixtureInfo, FixtureMixerOutput, patcher } from '@/scripts/api/ipc';

	let patcherState = usePatcherState();
	let liveMix = useLiveMixState();

	const canvas = ref<HTMLCanvasElement | null>(null);
	const vis = ref<Canvas | null>(null);
	const viewport = ref<HTMLDivElement | null>(null);
	const viewportBounds = useElementBounding(viewport);

	/** Map of fixture IDs to fabric objects */
	const fixtures = reactive(new Map<string, Circle>());

	/**
	 * Map of fabric objects to fixture IDs.
	 * Weakly typed due to the number of incompatible fabric object types
	 */
	const fixtureObjToId = new WeakMap<any, string>();

	watch(liveMix, () => {
		if (!liveMix.value) return;

		// Fixtures should be created/destroyed by patcher state
		for (const fixtureId of fixtures.keys()) {
			if (!liveMix.value[fixtureId]) continue;
			updateLight(fixtureId);
		}
		if (vis.value) vis.value.requestRenderAll();
	}, { immediate: true });

	function getActiveControlGroups(fixtureId: string): ControlGroup[] | null {
		// Collect data and drop unsynchronized triggers
		if (
			!patcherState.value
			|| !vis.value
			|| !fixtures.has(fixtureId)
			|| !liveMix.value
		) return null;

		let fixtureInstance = patcherState.value.fixtures[fixtureId];
		if (!fixtureInstance) return null;
		let fixtureTypeInfo = patcherState.value.library[fixtureInstance.fixture_id];
		if (!fixtureTypeInfo) return null;
		let fixturePersonalityInfo = fixtureTypeInfo.personalities[fixtureInstance.personality];
		if (!fixturePersonalityInfo) return null;
		let fixtureData = liveMix.value[fixtureId];
		if (!fixtureData) return null;
		let fixtureVis = fixtures.get(fixtureId);
		if (!fixtureVis) return null;
		let availableChannels = fixturePersonalityInfo.available_channels;

		const activeControlGroups = fixtureTypeInfo.control_groups.filter((group) => exhaustiveMatch(group, {
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

	function includesAll<T>(source: T[], criteria: T[]) {
		return !criteria.some((item) => !source.includes(item));
	}

	function cmyk2rgb(
		cyan: number,
		magenta: number,
		yellow: number,
		black: number,
	): { red: number, green: number, blue: number } {
		return {
			red: 255 * (1 - cyan) * (1 - black),
			green: 255 * (1 - magenta) * (1 - black),
			blue: 255 * (1 - yellow) * (1 - black),
		};
	}

	/**
	 * Normalizes a channel into 8-bit precision
	 */
	function normalizeChannel(
		profile: FixtureInfo,
		fixtureData: FixtureMixerOutput,
		channel: string,
	) {
		const channelValue = fixtureData[channel];
		const channelInfo = profile.channels[channel];
		return exhaustiveMatch(channelInfo.size, {
			U8: () => channelValue,
			U16: () => Math.floor(channelValue / 257), // Maps 65535 to 255
		});
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
			|| !liveMix.value
		) return;

		let fixtureInstance = patcherState.value.fixtures[fixtureId];
		if (!fixtureInstance) return;
		let fixtureTypeInfo = patcherState.value.library[fixtureInstance.fixture_id];
		if (!fixtureTypeInfo) return;
		let fixturePersonalityInfo = fixtureTypeInfo.personalities[fixtureInstance.personality];
		if (!fixturePersonalityInfo) return;
		let fixtureData = liveMix.value[fixtureId];
		if (!fixtureData) return;
		let fixtureVis = fixtures.get(fixtureId);
		if (!fixtureVis) return;

		// Get data bounds
		let intensityValue = 0;
		let redValue = 0;
		let greenValue = 0;
		let blueValue = 0;
		let controlGroups = getActiveControlGroups(fixtureId);

		if (controlGroups) controlGroups.forEach((group) => exhaustiveMatch(group, {
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
		vis.value.on("selection:updated", (event) => selected.value = event.selected);
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

		// vis.value.add(new Circle({
		// 	stroke: "#FFFFFF",
		// 	radius: 15,
		// 	strokeWidth: 1,
		// 	fill: createGradient(1, 255, 100, 0),
		// 	top: 0,
		// 	left: 0,
		// 	hasControls: false,
		// }));
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
		}
	}
</style>
