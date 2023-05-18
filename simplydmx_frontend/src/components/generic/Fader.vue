<script lang="ts" setup>
	import { ref, watch, computed, onBeforeUnmount } from "vue";
	import { useElementBounding } from "@vueuse/core";

	const props = defineProps<{
		label?: string,
		modelValue: number,
	}>();

	const emit = defineEmits<{
		(event: "update:modelValue", value: number): void;
	}>();

	watch(() => props.modelValue, () => {
		if (!mouseDown.value && activeTouch.value === null && scrollTimeout === null) {
			intermediateValue.value = props.modelValue;
		}
	});

	const fader = ref<HTMLDivElement>();
	const intermediateValue = ref(props.modelValue);
	const visibleValue = computed(() => {
		if (intermediateValue.value > 1) return 1;
		if (intermediateValue.value < 0) return 0;
		return intermediateValue.value;
	});
	const faderBounds = useElementBounding(fader);

	let scrollTimeout: number | null = null;
	function handleScroll(event: WheelEvent) {
		const beforeValue = visibleValue.value;
		intermediateValue.value += event.deltaY / 350;
		intermediateValue.value = visibleValue.value;
		if (visibleValue.value !== beforeValue) emit("update:modelValue", visibleValue.value);
		if (scrollTimeout !== null) clearTimeout(scrollTimeout);
		scrollTimeout = setTimeout(() => {
			intermediateValue.value = props.modelValue;
			clearTimeout(scrollTimeout!);
			scrollTimeout = null;
		}, 700);
	}

	let mouseDown = ref(false);
	let mouseY = 0;
	function handleMouseDown(event: MouseEvent) {
		mouseDown.value = true;
		mouseY = event.screenY;
		event.preventDefault();
		document.addEventListener("mouseup", handleMouseUp);
		document.addEventListener("mousemove", handleMouseMove);
	}
	function handleMouseMove(event: MouseEvent) {
		if (mouseDown.value) {
			event.preventDefault();
			let delta = mouseY - event.screenY;
			mouseY = event.screenY;
			if (delta !== 0) {
				const beforeValue = visibleValue.value;
				intermediateValue.value += delta / faderBounds.height.value;
				if (visibleValue.value !== beforeValue) emit("update:modelValue", visibleValue.value);
			}
		}
	}
	function handleMouseUp(event: MouseEvent) {
		mouseDown.value = false;
		intermediateValue.value = props.modelValue;
		event.preventDefault();
		document.removeEventListener("mouseup", handleMouseUp);
		document.removeEventListener("mousemove", handleMouseMove);
	}

	const activeTouch = ref<number | null>(null);
	let touchY = 0;
	function handleTouchStart(event: TouchEvent) {
		// Track first touch down/up
		if (event.type === "touchstart" && activeTouch.value === null && event.changedTouches.length) {
			event.preventDefault();
			const touch = event.changedTouches[0];
			activeTouch.value = touch.identifier;
			touchY = touch.screenY;
			document.addEventListener("touchmove", handleTouchStart);
		} else if (event.type === "touchend" && activeTouch.value !== null && event.changedTouches.length) {
			for (let i = 0; i < event.changedTouches.length; i++) {
				const touch = event.changedTouches[i];
				if (touch.identifier === activeTouch.value) {
					event.preventDefault();
					intermediateValue.value = props.modelValue;
					activeTouch.value = null;
					document.removeEventListener("touchmove", handleTouchStart);
					break;
				}
			}
		} else if (event.type === "touchmove" && activeTouch.value !== null) {
			// Look for the touch that was intended for us
			let touch: Touch | null = null;
			for (let i = 0; i < event.changedTouches.length; i++) {
				const thisTouch = event.changedTouches[i];
				if (thisTouch.identifier === activeTouch.value) {
					touch = thisTouch;
					break;
				}
			}
			if (!touch) return;

			// Do something with it
			event.preventDefault();
			let delta = touchY - touch.screenY;
			touchY = touch.screenY;
			if (delta !== 0) {
				const beforeValue = visibleValue.value;
				intermediateValue.value += delta / faderBounds.height.value;
				if (visibleValue.value !== beforeValue) emit("update:modelValue", visibleValue.value);
			}
		}
	}

	onBeforeUnmount(() => {
		document.removeEventListener("mouseup", handleMouseUp);
		document.removeEventListener("mousemove", handleMouseMove);
		document.removeEventListener("touchmove", handleTouchStart);
	});
</script>

<template>
	<div
		ref="fader"
		:class="{
			'sdmx-control-fader': true,
			'active-touch': activeTouch !== null,
		}"
		:style="{
			'--fader-percentage': (visibleValue * 100) + '%',
		}"
		@wheel="handleScroll"
		@mousedown="handleMouseDown"
		@touchstart="handleTouchStart"
		@touchmove="handleTouchStart"
		@touchend="handleTouchStart"
		>
		<div v-if="props.label" class="sdmx-control-fader__contents">
			{{ props.label }}
		</div>
	</div>
</template>

<style lang="scss">
	.sdmx-control-fader {
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

		// transition: scale 300ms cubic-bezier(0.68, -0.6, 0.32, 1.6);
		transition: scale 200ms ease-in-out;
		&.active-touch {
			scale: 0.98;
		}

		&:after {
			content: "";
			position: absolute;
			bottom: 0;
			left: 0;
			right: 0;
			background-color: #DDDDDDDD;
			height: var(--fader-percentage);
			cursor: pointer;
		}

		.sdmx-control-fader__contents {
			z-index: 1;
			color: #555;
			// text-shadow: #FFF 0 0 3px;
			pointer-events: none;
			-webkit-pointer-events: none;
			user-select: none;
			-webkit-user-select: none;
		}
	}
</style>
