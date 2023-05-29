<script lang="ts" setup>
	import { computed, ref } from "vue";
	import { usePatcherState } from "@/stores/patcher";
	import CreateFixtureDialog from "./CreateFixtureDialog.vue";
	import FixtureEditor from "./FixtureEditor.vue";
	import { patcher } from "@/scripts/api/ipc";

	const state = usePatcherState();
	const fixtureOptions = computed(() => {
		if (!state.value) return [];
		return state.value.fixture_order.map((value) => {
			const fixture = state.value!.fixtures[value];
			return {
				name: fixture.name || fixture.id,
				value: fixture.id,
			};
		});
	});

	const selectedFixture = ref(null);
	const addFixtureDialog = ref(false);
	const deleteFixtureConfirm = ref<string | null>(null);

	const deletingFixtureName = computed(() => {
		if (deleteFixtureConfirm.value === null) return null;
		return fixtureOptions.value?.find((item) => item.value === deleteFixtureConfirm.value)?.name || null;
	});

	async function deleteQueuedFixture() {
		if (deleteFixtureConfirm.value) {
			await patcher.delete_fixture(deleteFixtureConfirm.value);
			deleteFixtureConfirm.value = null;
		}
	}
</script>

<template>
	<div class="patcher-add">
		<LargeSelect
			class="patcher-left-sidebar"
			v-model="selectedFixture"
			:options="fixtureOptions"
			enable-search
			>
			<template #header-right>
				<Tooltip text="Add Fixture">
					<Button @click="addFixtureDialog = true" icon subtle><Icon i="plus"/></Button>
				</Tooltip>
			</template>
			<template #option="{ option }">
				{{ option.name }}
				<div v-if="selectedFixture === option.value" class="delete-button">
					<Tooltip text="Delete fixture" placement="right">
						<Button subtle icon @click="deleteFixtureConfirm = option.value">
							<Icon class="delete-icon" i="trashCan" />
						</Button>
					</Tooltip>
				</div>
			</template>
		</LargeSelect>
		<div class="patcher-fixture-prefs">
			<FixtureEditor v-if="selectedFixture" :selectedFixture="selectedFixture" />
		</div>
		<CreateFixtureDialog v-model:visible="addFixtureDialog" />
		<Dialog :visible="deleteFixtureConfirm != null">
			<template #header>
				Delete fixture?
			</template>
			Are you sure you want to delete the fixture &quot;{{ deletingFixtureName }}&quot;?<br><br>
			<span style="opacity: 0.75">All data currently associated with this fixture will be deleted.</span>
			<template #footer>
				<Button @click="deleteFixtureConfirm = null" class="spaced">
					No
				</Button>
				<div class="spacer"/>
				<Button @click="deleteQueuedFixture()" class="spaced">
					Yes
				</Button>
			</template>
		</Dialog>
	</div>
</template>

<style lang="scss">
	.patcher-add {
		width: 100%;
		height: 100%;
		
		display: flex;
		flex-flow: row nowrap;

		.patcher-left-sidebar {
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

		.patcher-fixture-prefs {
			flex-grow: 1;

			display: flex;
			flex-flow: column nowrap;
		}
	}
</style>
