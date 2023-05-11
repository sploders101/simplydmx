<script lang="ts" setup>
	import { VisibleControlGroup } from "../types";

	const props = defineProps<{
		group: VisibleControlGroup,
	}>();
</script>

<template>
	<div class="sdmx-fixture-control">
		{{ props.group.name }}
	</div>
</template>

<style lang="scss">
	.sdmx-fixture-control {
		background-color: red;
		border: 2px solid black;
		min-width: 5rem;

		text-align: center;
		display: flex;
		flex-flow: column nowrap;
		justify-content: center;
	}
</style>
