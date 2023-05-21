<script lang="ts" setup>
	import { computed } from "vue";
	import { VisibleControlGroup } from "../types";
	import Fader from "@/components/generic/Fader.vue";
	import { exhaustiveMatch, FullMixerOutput, SubmasterData } from "@/scripts/api/ipc";
	import { normalizeChannel } from "@/scripts/conversions";
	import { usePatcherState } from "@/stores/patcher";

	const props = defineProps<{
		displayData: FullMixerOutput | SubmasterData,
		group: VisibleControlGroup<"fader">,
	}>();
	const emit = defineEmits<{
		(event: "update-props", props: SubmasterData): void,
	}>();

	const patcherState = usePatcherState();

	const faderValue = computed(() => {
		if (!patcherState.value) return 0;
		let summed = 0;
		props.group.controls.forEach((control) => {
			const profile = patcherState.value?.library[patcherState.value!.fixtures[control.instanceId].fixture_id]!;
			summed += normalizeChannel(
				profile,
				props.displayData[control.instanceId],
				exhaustiveMatch(control.controlData, {
					Intensity: (channel) => channel,
					Zoom: (channel) => channel,
					GenericInput: (channel) => channel,
				}),
				"percentage",
			);
		});
		return summed / props.group.controls.length;
	});

	function updateChannels(newValue: number) {
		const delta: SubmasterData = {};
		props.group.controls.forEach((control) => {
			const profile = patcherState.value?.library[patcherState.value!.fixtures[control.instanceId].fixture_id]!;
			if (!delta[control.instanceId]) delta[control.instanceId] = {};
			const fixtureDelta = delta[control.instanceId];
			const normalizeValue = (channel: string) => Math.floor(newValue * exhaustiveMatch(profile.channels[channel].size, {
				U8: () => 255,
				U16: () => 65535,
			}));
			exhaustiveMatch(control.controlData, {
				Intensity: (channel) => fixtureDelta[channel] = { Static: normalizeValue(channel) },
				Zoom: (channel) => fixtureDelta[channel] = { Static: normalizeValue(channel) },
				GenericInput: (channel) => fixtureDelta[channel] = { Static: normalizeValue(channel) },
			});
		});
		emit("update-props", delta);
	}
</script>

<template>
	<Fader
		:label="props.group.name"
		:modelValue="faderValue"
		@update:modelValue="updateChannels"
		/>
</template>

<style lang="scss">

</style>