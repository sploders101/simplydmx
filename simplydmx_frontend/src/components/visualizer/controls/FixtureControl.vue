<script lang="ts" setup>
	import { ref } from "vue";
	import { VisibleControlGroup } from "../types";
	import Fader from "./Fader.vue";
	import Debug from "./Debug.vue";

	const props = defineProps<{
		group: VisibleControlGroup,
	}>();

	const faderValue = ref(0.5);
	function updateModelValue(event: number) {
		setTimeout(() => {
			console.log("Setting value to " + event);
			faderValue.value = event;
		}, 1000);
	}
</script>

<template>
	<Fader
		v-if="props.group.type === 'fader'"
		:modelValue="faderValue"
		@update:modelValue="updateModelValue($event)"
		label="Intensity"
		/>
	<Debug v-else :group="props.group" />
</template>
