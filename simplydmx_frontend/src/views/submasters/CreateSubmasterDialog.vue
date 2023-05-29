<script lang="ts" setup>
	import { ref } from "vue";
	import { mixer } from "@/scripts/api/ipc";


	const props = defineProps<{
		visible: boolean;
	}>();
	const emit = defineEmits<
		(e: "update:visible", visible: boolean) => void
	>();


	function cancelAddingSubmaster() {
		emit("update:visible", false);
		name.value = "";
	}

	const name = ref<string>("");

	/** Adds the submaster  */
	async function addSubmaster() {
		if (name.value) {
			await mixer.create_layer(name.value);
			emit("update:visible", false);
		} else {
			alert("Invalid form details");
		}
	}
</script>

<template>
	<Dialog :visible="props.visible" :show-close="false">
		<template #header>
			Add Submaster
		</template>

		<Textbox
			label="Name"
			v-model="name"
			class="spaced"
			/>

		<template #footer>
			<Button @click="cancelAddingSubmaster()" subtle class="spaced">
				Cancel
			</Button>
			<div class="spacer"/>
			<Button @click="addSubmaster()" class="spaced">
				Ok
			</Button>
		</template>
	</Dialog>
</template>
