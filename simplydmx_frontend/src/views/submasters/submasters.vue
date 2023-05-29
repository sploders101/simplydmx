<script lang="ts" setup>
	import { ref } from "vue";
	import { useTypeSpecState } from "@/stores/typespec";
	import { useSubmasterData } from "@/stores/submaster";
	import * as ipc from "@/scripts/api/ipc/rpc";
	import Visualizer from "@/components/visualizer/visualizer.vue";

	const submasterOptions = useTypeSpecState("submasters");

	const selectedSubmaster = ref(null);
	const submasterData = useSubmasterData(() => selectedSubmaster.value);

	const addSubmasterDialog = ref(false);

	async function updateSubmaster(delta: ipc.SubmasterData) {
		if (!selectedSubmaster.value) return;
		await ipc.mixer.set_layer_contents(selectedSubmaster.value, delta);
	}
</script>

<template>
	<div class="submaster-editor">
		<LargeSelect
			class="submaster-editor__left-sidebar"
			v-model="selectedSubmaster"
			:options="submasterOptions || []"
			enable-search
			>
			<template #header-right>
				<Tooltip text="Create Submaster">
					<Button @click="addSubmasterDialog = true" icon subtle><Icon i="plus"/></Button>
				</Tooltip>
			</template>
		</LargeSelect>
		<div class="submaster-editor__visualizer">
			<Visualizer v-if="submasterData" :display-data="submasterData" :update-props="updateSubmaster"/>
		</div>
	</div>
</template>

<style lang="scss">
	.submaster-editor {
		width: 100%;
		height: 100%;
		
		display: flex;
		flex-flow: row nowrap;

		.submaster-editor__left-sidebar {
		    height: 100%;
		    max-width: 20rem;
		    min-width: 15rem;
		    width: 25%;
		}

		.submaster-editor__visualizer {
			flex-grow: 1;

			display: flex;
			flex-flow: column nowrap;
			overflow: auto;
		}
	}
</style>
