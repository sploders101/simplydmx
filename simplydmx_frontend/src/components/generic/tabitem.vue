<script lang="ts" setup>
	import { inject, type Ref } from "vue";

	const props = defineProps<{
		tab: string,
	}>();
	const tabId = inject<Ref<string>>("sdmx-tabs__current-tab-id");
</script>

<template>
	<div class="sdmx-tab-container" v-if="tabId == props.tab">
		<slot/>
	</div>
</template>

<style lang="scss">
	.sdmx-tab-container {
		width: 100%;
		height: 100%;
	}
</style>
