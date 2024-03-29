<script lang="ts" setup>
	import { computed, ref } from "vue";
	import CreateUniverseDialog from "./CreateUniverseDialog.vue";
	import UniverseEditor from "./UniverseEditor.vue";
	import { useTypeSpecState } from "@/stores/typespec";
	import { output_dmx } from "@/scripts/api/ipc/rpc";

	const universeOptions = useTypeSpecState("universes");

	const selectedUniverse = ref(null);
	const addUniverseDialog = ref(false);
	const deleteUniverseConfirm = ref<string | null>(null);

	const deletingUniverseName = computed(() => {
		if (deleteUniverseConfirm.value === null) return null;
		return universeOptions.value?.find((item) => item.value === deleteUniverseConfirm.value)?.name || null;
	});

	async function deleteQueuedUniverse() {
		if (deleteUniverseConfirm.value) {
			await output_dmx.delete_universe(deleteUniverseConfirm.value);
			deleteUniverseConfirm.value = null;
		}
	}
</script>

<template>
	<div class="patcher-universes">
		<LargeSelect
			class="patcher-universes-left-sidebar"
			v-model="selectedUniverse"
			:options="universeOptions || []"
			enable-search
			>
			<template #header-right>
				<Tooltip text="Add Universe">
					<Button @click="addUniverseDialog = true" icon subtle><Icon i="plus"/></Button>
				</Tooltip>
			</template>
			<template #option="{ option }">
				{{ option.name }}
				<div v-if="selectedUniverse === option.value" class="delete-button">
					<Tooltip text="Delete universe" placement="right">
						<Button subtle icon @click="deleteUniverseConfirm = option.value">
							<Icon class="delete-icon" i="trashCan" />
						</Button>
					</Tooltip>
				</div>
			</template>
		</LargeSelect>
		<div class="patcher-universes-prefs">
			<UniverseEditor v-if="selectedUniverse" :selectedUniverse="selectedUniverse" />
		</div>
		<CreateUniverseDialog v-model:visible="addUniverseDialog" />
		<Dialog :visible="deleteUniverseConfirm != null">
			<template #header>
				Delete universe?
			</template>
			Are you sure you want to delete the universe &quot;{{ deletingUniverseName }}&quot;?<br><br>
			<span style="opacity: 0.75">Any fixtures currently associated with this universe will be unlinked.</span>
			<template #footer>
				<Button @click="deleteUniverseConfirm = null" class="spaced">
					No
				</Button>
				<div class="spacer"/>
				<Button @click="deleteQueuedUniverse()" class="spaced">
					Yes
				</Button>
			</template>
		</Dialog>
	</div>
</template>

<style lang="scss">
	.patcher-universes {
		width: 100%;
		height: 100%;
		
		display: flex;
		flex-flow: row nowrap;

		.patcher-universes-left-sidebar {
		    height: 100%;
		    max-width: 20rem;
		    min-width: 15rem;
		    width: 25%;

			.delete-button {
				margin-left: auto;

				.delete-icon {
					fill: #AA0000;
				}
			}
		}

		.patcher-universes-prefs {
			flex: 1 1 0;
			overflow: auto;

			display: flex;
			flex-flow: column nowrap;
		}
	}
</style>
