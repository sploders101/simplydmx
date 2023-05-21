<script lang="ts" setup>
	import { ref, watch } from "vue";

	const props = defineProps<{
		label?: string,
		modelValue: string,
	}>();

	const emit = defineEmits<{
		(event: "update:modelValue", value: string): void;
	}>();

	const intermediateValue = ref(props.modelValue);
	watch(() => props.modelValue, () => {
		if (!locked.value) {
			intermediateValue.value = props.modelValue;
		}
	});

	// input event locks; change event unlocks to prevent jitter
	const locked = ref(false);

	function changeValue(isInput: boolean, newValue: string) {
		locked.value = isInput;
		if (intermediateValue.value !== newValue) {
			intermediateValue.value = newValue;
			emit("update:modelValue", newValue);
		}
	}
</script>

<template>
	<div
		class="sdmx-control-color"
		:style="{ 'background-color': props.modelValue }"
		>
		<input
			type="color"
			class="sdmx-control-color__native"
			:value="intermediateValue"
			@input="changeValue(true, ($event.target as any).value)"
			@change="changeValue(false, ($event.target as any).value)"
			>
		<div v-if="props.label" class="sdmx-control-color__contents">
			{{ props.label }}
		</div>
	</div>
</template>

<style lang="scss">
	.sdmx-control-color {
		background-color: #FFFFFF20;
		width: 5rem;
		border-radius: 0.5rem;

		text-align: center;
		display: flex;
		flex-flow: column nowrap;
		justify-content: flex-end;
		padding-bottom: 0.5rem;
		position: relative;
		overflow: hidden;

		* {
			cursor: pointer !important;
		}

		.sdmx-control-color__native {
			opacity: 0;
			position: absolute;
			top: 0;
			left: 0;
			width: 100%;
			height: 100%;
			margin: 0;
			padding: 0;
			cursor: default !important;
		}

		.sdmx-control-color__contents {
			z-index: 1;
			color: #555;
			text-shadow: #00000055 0 0 5px;
			pointer-events: none;
			-webkit-pointer-events: none;
			user-select: none;
			-webkit-user-select: none;
		}
	}
</style>
