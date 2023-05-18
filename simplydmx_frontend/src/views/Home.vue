<script lang="ts" setup>
	import { ref } from "vue";
	import { unwrap } from "@/scripts/helpers";
	import * as rpc from "@/scripts/api/ipc";
	import * as mixer from "@/scripts/api/mixer";
	import * as patcher from "@/scripts/api/patcher";
	import { usePatcherState } from "@/stores/patcher";
	import { Channel, ChannelSize, SubmasterData } from "@/scripts/api/ipc";
	import { useTypeSpecState } from "@/stores/typespec";
	import Visualizer from "@/components/visualizer/visualizer.vue";
	import { useLiveMixState } from "@/stores/live";

	const liveMix = useLiveMixState();
	const patcherState = usePatcherState();

	const log = console.log.bind(console);

	const test = ref("Testing");
	const testNumber = ref(5);
	const test2 = ref("test1");
	const options = [
		{
			name: "Test 1",
			value: "test1",
		},
		{
			name: "Test 2",
			value: "test2",
		},
		{
			name: "Test 3",
			value: "test3",
		},
		{
			name: "This is a test of a really long name that probably won't fit in the width that has been allocated to the dropdown",
			value: "test4",
		},
	]

	function testButton() {
		if (patcherState.value && patcherState.value.fixture_order.length === 0) {
			fullTestInvoked().catch((err) => {
				console.error(err);
			});
			dialogVisible.value = true;
		}
	}

	const dialogVisible = ref(false);
	const selectValue = ref(1);

	(window as any).rpc = rpc;
	(window as any).mixer = mixer;
	(window as any).patcher = patcher;

	function blenderValue(value: number): rpc.BlenderValue {
		return {
			Static: value,
		};
	}

	// Examples
	const colorChannel = (size: ChannelSize): Channel => ({
		intensity_emulation: null,
		size: size,
		ch_type: {
			Linear: {
				priority: "LTP",
			}
		},
	});

	async function importFixtureTest(id: string, name: string, shortName: string, color: boolean, panTilt: boolean) {
		const importResponse = await rpc.patcher.import_fixture({
			fixture_info: {
				id,
				name,
				short_name: shortName,
				manufacturer: "Generic",
				family: "Generic",
				metadata: {
					manual_link: null,
					manufacturer: null,
				},
				channels: {
					...color ? {
						intensity: {
							intensity_emulation: ["red", "green", "blue", "white"],
							size: "U8",
							ch_type: {
								Linear: {
									priority: "HTP",
								},
							},
						},
						intensity16: {
							intensity_emulation: ["red16", "green16", "blue16", "white16"],
							size: "U16",
							ch_type: {
								Linear: {
									priority: "HTP",
								},
							},
						},
						red: colorChannel("U8"),
						green: colorChannel("U8"),
						blue: colorChannel("U8"),
						white: colorChannel("U8"),
						red16: colorChannel("U16"),
						green16: colorChannel("U16"),
						blue16: colorChannel("U16"),
						white16: colorChannel("U16"),
					} : {
						intensity: {
							intensity_emulation: null,
							size: "U8",
							ch_type: {
								Linear: {
									priority: "HTP",
								},
							},
						},
						intensity16: {
							intensity_emulation: null,
							size: "U16",
							ch_type: {
								Linear: {
									priority: "HTP",
								},
							},
						},
					},
					...panTilt ? {
						pan: colorChannel("U8"),
						tilt: colorChannel("U8"),
						pan16: colorChannel("U16"),
						tilt16: colorChannel("U16"),
					} : {},
				},
				personalities: {
					"8-bit": {
						available_channels: ["intensity", ...color ? ["red", "green", "blue", "white"] : [], ...panTilt ? ["pan", "tilt"] : []],
					},
					"16-bit": {
						available_channels: ["intensity16", ...color ? ["red16", "green16", "blue16", "white16"] : [], ...panTilt ? ["pan16", "tilt16"] : []],
					},
				},
				output_driver: "DMX",
				control_groups: [
					{ name: null, channels: { Intensity: "intensity" } },
					{ name: null, channels: { Intensity: "intensity16" } },
					...color ? [
						{ name: null, channels: { RGBGroup: { red: "red", green: "green", blue: "blue" } } },
						{ name: null, channels: { RGBGroup: { red: "red16", green: "green16", blue: "blue16" } } },
					] : [],
					...panTilt ? [
						{ name: null, channels: { PanTilt: { pan: "pan", tilt: "tilt" } } },
						{ name: null, channels: { PanTilt: { pan: "pan16", tilt: "tilt16" } } },
					] : [],
				],
			},
			output_info: {
				personalities: {
					"8-bit": {
						dmx_channel_order: [...color ? ["red", "green", "blue", "white"] : [], ...panTilt ? ["pan", "tilt"] : []],
					},
					"16-bit": {
						dmx_channel_order: [...color ? ["red16", "green16", "blue16", "white16"] : [], ...panTilt ? ["pan16", "tilt16"] : []],
					},
				},
			},
		});
	}

	async function fullTestInvoked() {
		await importFixtureTest("c205635c-037a-4e5c-8a68-59a8a86dae8f", "Generic RGBW Fixture", "RGBW", true, false);
		await importFixtureTest("ce57c131-cc29-4088-84d2-1472b73f7d65", "Generic RGBWPT Fixture", "RGBWPT", true, true);
		await importFixtureTest("bb5a93f8-4e26-422f-9ac8-5b9a15a9254d", "Generic Spotlight", "I", false, false);
		await importFixtureTest("a93e6ba5-83ef-4faa-afd0-7b85dd12400b", "Generic Moving Spotlight", "IPT", false, true);

		let universeId = await rpc.output_dmx.create_universe("Test universe");
		let fixtureId = unwrap(await rpc.patcher.create_fixture("c205635c-037a-4e5c-8a68-59a8a86dae8f", "8-bit", "RGBW", null, {
			universe: universeId,
			offset: 41,
		} as rpc.DMXFixtureInstance));

		let submasterId = await rpc.mixer.create_layer("Example submaster");
		let newContents = {
			[fixtureId]: {
				intensity: blenderValue(255),
				red: blenderValue(255),
				green: blenderValue(30),
				blue: blenderValue(0),
				white: blenderValue(0),
			}
		};
		await rpc.mixer.set_layer_contents(submasterId, newContents);
		await rpc.mixer.set_layer_opacity(submasterId, 65535, true);
	}

	const typespec = useTypeSpecState("submasters");
	const test3 = ref("");

	function logVisualizerChanges(changes: SubmasterData) {
		return new Promise<void>((res) => {
			setTimeout(() => {
				console.log(changes);
				res();
			}, 500);
		});
	}

</script>

<template>
	<Tabs :tabs="[{label:'Test1',id:'test1'}, {label:'Test2',id:'test2'}, {label:'Test3',id:'test3'}, {label:'Test4',id:'test4'}]">
		<Tabitem tab="test1">
			<p>
				This is currently my testing ground for all of the UI elements in SimplyDMX. Anything with functionality
				will be demonstrated here using a component. By isolating all user interactivity problems into a set of
				components, I get reusability and upgradability. For example, if I need a custom dropdown for a specific
				part of the app to augment its function, it will be defined first in the dropdown component, then used
				here, then used in the part of the app where I need it. By architecting user-interactivity in this way,
				I can easily add more input devices later like UI control via MIDI, or even more useful, touch for the
				eventual mobile build (This <strong><i>is</i></strong> coming; I've already done it, it just needs to
				be brought up to date with the new APIs and needs a bit more work to be stable. I'm also waiting on an
				official build of Wry that is declared as stable on mobile).
			</p>
			<p>{{ test }}</p>
			<p>{{ test2 }}</p>
			<p>{{ JSON.stringify(typespec) }}</p>
			<Textbox v-model="test" class="spaced" />
			<NumberInput v-model="testNumber" class="spaced" />
			<Dropdown v-model="test2" :options="options" class="spaced" />
			<Dropdown v-model="test3" :options="typespec || []" class="spaced" />
			<Button @click="testButton()" class="spaced">Run initialization test</button>
		</Tabitem>
		<Tabitem tab="test2">
			<LargeSelect :options="[1, 2, 3, 4, 5].map((i) => ({ name: 'Test ' + i, value: i}))" v-model="selectValue" enableSearch />
		</Tabitem>
		<Tabitem tab="test3">
			<LargeSelect :options="[1, 2, 3, 4, 5].map((i) => ({ name: 'Test ' + i, value: i}))" @select="log" enableSearch>
				<template #header-right>
					<Tooltip text="Add Fixture">
						<Button icon subtle><Icon i="plus"/></Button>
					</Tooltip>
				</template>
			</LargeSelect>
		</Tabitem>
		<Tabitem tab="test4">
			<Visualizer :display-data="liveMix" :update-props="logVisualizerChanges"/>
		</Tabitem>
	</Tabs>

	<Dialog v-model:visible="dialogVisible">
		<template #header>
			Test Ran
		</template>

		The initialization test has been run. This test cannot be run again until the app has been restarted, as it may yield
		unexpected results. This will not be needed in the future as more UI elements are implemented.

		<template #footer>
			<Button @click="dialogVisible = false" subtle class="spaced">
				Cancel
			</Button>
			<div class="spacer"/>
			<Button @click="dialogVisible = false" class="spaced">
				Ok
			</Button>
		</template>
	</Dialog>
</template>

<style lang="scss">
</style>
