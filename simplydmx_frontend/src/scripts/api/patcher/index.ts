import {
	listen,
	patcher,
	VisualizationInfo,
	FixtureInstance,
	FixtureInfo,
} from "../ipc";
import type {
	SharablePatcherState,
} from "../ipc";


export async function listenForUpdates(mutableGetter: () => SharablePatcherState | null, setter: (state: SharablePatcherState) => void): Promise<() => Promise<void>> {
	const patchUpdates = listen<FixtureInstance | null>(
		"patcher.patch_updated",
		{ type: "None" },
		(event) => {
			if (event.criteria.type !== "Uuid") throw new Error("Invalid criteria");
			const existingState = mutableGetter();
			if (existingState !== null) {
				existingState.fixture_order.push(event.criteria.data);
				if (event.data === null) {
					delete existingState.fixtures[event.criteria.data];
				} else {
					existingState.fixtures[event.criteria.data] = event.data;
				}
			}
		},
	);
	const libraryUpdates = listen<FixtureInfo>(
		"patcher.new_fixture",
		{ type: "None" },
		(event) => {
			const existingState = mutableGetter();
			if (existingState !== null) {
				existingState.library[event.data.id] = event.data;
			}
		},
	);
	const visualizationUpdates = listen<VisualizationInfo>(
		"patcher.visualization_updated",
		{ type: "None" },
		(event) => {
			const existingState = mutableGetter();
			if (event.criteria.type !== "Uuid") throw new Error("Invalid criteria");
			if (existingState && existingState.fixtures[event.criteria.data]) {
				existingState.fixtures[event.criteria.data].visualization_info = event.data;
			} else {
				patcher.get_patcher_state().then(setter);
			}
		},
	);

	const stopPatchUpdates = await patchUpdates;
	const stopLibraryUpdates = await libraryUpdates;
	const stopVisualizationUpdates = await visualizationUpdates;

	return () => Promise.all([
		stopPatchUpdates(),
		stopLibraryUpdates(),
		stopVisualizationUpdates(),
	]).then(() => {});
}
