<script lang="ts" setup>
	import { computed, ref } from "vue";
	import { useTypeSpecState } from "@/stores/typespec";
	import { useSubmasterData } from "@/stores/submaster";
	import * as ipc from "@/scripts/api/ipc/rpc";
	import Visualizer from "@/components/visualizer/visualizer.vue";
	import CreateSubmasterDialog from "./CreateSubmasterDialog.vue";
	import Textbox from "@/components/generic/textbox.vue";

	const submasterOptions = useTypeSpecState("submasters");

	const selectedSubmaster = ref(null);
	const submasterData = useSubmasterData(() => selectedSubmaster.value);

	const createSubmasterDialog = ref(false);
	const deleteSubmasterConfirm = ref<string | null>(null);
	const renameSubmaster = ref<string | null>(null);
	const newSubmasterName = ref<string>("");

	const deletingSubmasterName = computed(() => {
		if (deleteSubmasterConfirm.value === null) return null;
		return submasterOptions.value?.find((item) => item.value === deleteSubmasterConfirm.value)?.name || null;
	});

	async function deleteQueuedSubmaster() {
		if (deleteSubmasterConfirm.value) {
			await ipc.mixer.delete_layer(deleteSubmasterConfirm.value);
			deleteSubmasterConfirm.value = null;
		}
	}

	async function updateSubmaster(delta: ipc.SubmasterData) {
		if (!selectedSubmaster.value) return;
		await ipc.mixer.set_layer_contents(selectedSubmaster.value, delta);
	}

	async function renameQueuedSubmaster() {
		if (!renameSubmaster.value) return;
		await ipc.mixer.rename_layer(renameSubmaster.value, newSubmasterName.value);
		newSubmasterName.value = "";
		renameSubmaster.value = null;
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
					<Button @click="createSubmasterDialog = true" icon subtle><Icon i="plus"/></Button>
				</Tooltip>
			</template>
			<template #option="{ option }">
				{{ option.name }}
				<div v-if="selectedSubmaster === option.value" class="rename-button">
					<Tooltip text="Rename submaster" placement="right">
						<Button subtle icon @click="renameSubmaster = option.value; newSubmasterName = option.name">
							<Icon class="rename-icon" i="textCursor" />
						</Button>
					</Tooltip>
				</div>
				<div v-if="selectedSubmaster === option.value" class="delete-button">
					<Tooltip text="Delete submaster" placement="right">
						<Button subtle icon @click="deleteSubmasterConfirm = option.value">
							<Icon class="delete-icon" i="trashCan" />
						</Button>
					</Tooltip>
				</div>
			</template>
		</LargeSelect>
		<div class="submaster-editor__visualizer">
			<Visualizer v-if="submasterData" :display-data="submasterData" :update-props="updateSubmaster"/>
		</div>

		<CreateSubmasterDialog
			v-model:visible="createSubmasterDialog"
			/>

		<Dialog :visible="deleteSubmasterConfirm != null">
			<template #header>
				Delete submaster?
			</template>
			Are you sure you want to delete the submaster &quot;{{ deletingSubmasterName }}&quot;?<br><br>
			<span style="opacity: 0.75">All data currently associated with this submaster will be deleted.</span>
			<template #footer>
				<Button @click="deleteSubmasterConfirm = null" class="spaced">
					No
				</Button>
				<div class="spacer"/>
				<Button @click="deleteQueuedSubmaster()" class="spaced">
					Yes
				</Button>
			</template>
		</Dialog>

		<Dialog :visible="renameSubmaster != null">
			<template #header>
				Rename submaster
			</template>
			<Textbox v-model="newSubmasterName" />
			<template #footer>
				<Button @click="renameSubmaster = null" class="spaced">
					Cancel
				</Button>
				<div class="spacer"/>
				<Button @click="renameQueuedSubmaster()" class="spaced">
					Confirm
				</Button>
			</template>
		</Dialog>
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

			.rename-button {
				margin-left: auto;
				.rename-icon {
					opacity: 0.75;
				}
			}
			.delete-button {
				.delete-icon {
					fill: #AA0000;
				}
			}
		}

		.submaster-editor__visualizer {
			flex-grow: 1;

			display: flex;
			flex-flow: column nowrap;
			overflow: auto;
		}
	}
</style>
