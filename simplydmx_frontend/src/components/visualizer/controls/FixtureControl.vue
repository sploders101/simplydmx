<script lang="ts" setup>
	import { VisibleControlGroup } from "../types";
	import Fader from "./Fader.vue";
	import Debug from "./Debug.vue";
	import { FullMixerOutput, SubmasterData } from "@/scripts/api/ipc";

	const props = defineProps<{
		displayData: FullMixerOutput | SubmasterData,
		group: VisibleControlGroup,
	}>();
	const emit = defineEmits<{
		(event: "update-props", props: SubmasterData): void,
	}>();

</script>

<template>
	<Fader
		v-if="props.group.type === 'fader'"
		:display-data="props.displayData"
		:group="props.group"
		@update-props="emit('update-props', $event)"
		/>
	<Debug v-else :group="props.group" />
</template>
