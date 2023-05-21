<script lang="ts" setup>
	import { computed } from "vue";
	import { VisibleControlGroup } from "../types";
	import ColorPicker from "@/components/generic/ColorPicker.vue";
	import { exhaustiveMatch, FullMixerOutput, SubmasterData } from "@/scripts/api/ipc";
	import { cmyk2rgb, formatColorString, normalizeChannel, parseColorString, rgb2cmyk } from "@/scripts/conversions";
	import { usePatcherState } from "@/stores/patcher";

	const props = defineProps<{
		displayData: FullMixerOutput | SubmasterData,
		group: VisibleControlGroup<"color">,
	}>();
	const emit = defineEmits<{
		(event: "update-props", props: SubmasterData): void,
	}>();

	const patcherState = usePatcherState();

	const pickerValue = computed(() => {
		if (!patcherState.value) return "#000000";
		let summed = [0, 0, 0];
		props.group.controls.forEach((control) => {
			const profile = patcherState.value?.library[patcherState.value!.fixtures[control.instanceId].fixture_id]!;
			const displayData = props.displayData[control.instanceId];
			exhaustiveMatch(control.controlData, {
				RGBGroup: ({ red, green, blue }) => {
					summed[0] += normalizeChannel(profile, displayData, red);
					summed[1] += normalizeChannel(profile, displayData, green);
					summed[2] += normalizeChannel(profile, displayData, blue);
				},
				CMYKGroup: ({ cyan, magenta, yellow, black }) => {
					const { red, green, blue } = cmyk2rgb(
						normalizeChannel(profile, displayData, cyan),
						normalizeChannel(profile, displayData, magenta),
						normalizeChannel(profile, displayData, yellow),
						normalizeChannel(profile, displayData, black),
					);
					summed[0] += red;
					summed[1] += green;
					summed[2] += blue;
				},
			});
		});
		const test = formatColorString(
			summed[0] / props.group.controls.length,
			summed[1] / props.group.controls.length,
			summed[2] / props.group.controls.length,
		);
		return test;
	});

	const empty = Symbol();
	function runOnce<T>(func: () => T): () => T {
		let value: typeof empty | T = empty;
		return () => {
			if (value === empty) {
				value = func();
			}
			return value;
		};
	}

	function updateChannels(newColor: string) {
		const [redValue, greenValue, blueValue] = parseColorString(newColor);
		const cmykValues = runOnce(() => rgb2cmyk(redValue, greenValue, blueValue));

		const delta: SubmasterData = {};
		props.group.controls.forEach((control) => {
			const profile = patcherState.value?.library[patcherState.value!.fixtures[control.instanceId].fixture_id]!;
			if (!delta[control.instanceId]) delta[control.instanceId] = {};
			const fixtureDelta = delta[control.instanceId];

			const normalizeValue = (channel: string, value: number) => Math.floor(
				value * exhaustiveMatch(profile.channels[channel].size, {
					U8: () => 1,
					U16: () => 257,
				})
			);

			exhaustiveMatch(control.controlData, {
				RGBGroup: ({ red, green, blue }) => {
					fixtureDelta[red] = { Static: normalizeValue(red, redValue) };
					fixtureDelta[green] = { Static: normalizeValue(green, greenValue) };
					fixtureDelta[blue] = { Static: normalizeValue(blue, blueValue) };
				},
				CMYKGroup: ({ cyan, magenta, yellow, black }) => {
					const {
						cyan: cyanValue,
						magenta: magentaValue,
						yellow: yellowValue,
						black: blackValue,
					} = cmykValues();
					fixtureDelta[cyan] = { Static: normalizeValue(cyan, cyanValue) };
					fixtureDelta[magenta] = { Static: normalizeValue(magenta, magentaValue) };
					fixtureDelta[yellow] = { Static: normalizeValue(yellow, yellowValue) };
					fixtureDelta[black] = { Static: normalizeValue(black, blackValue) };
				},
			});
		});
		emit("update-props", delta);
	}
</script>

<template>
	<ColorPicker
		:label="props.group.name"
		:modelValue="pickerValue"
		@update:modelValue="updateChannels"
		/>
</template>

<style lang="scss">

</style>