import {
	listen,
	patcher,
	VisualizationInfo,
} from "../ipc";
import type {
	SharablePatcherState,
} from "../ipc";


export async function listenForUpdates(mutableGetter: () => SharablePatcherState | null, setter: (state: SharablePatcherState) => void): Promise<() => Promise<void>> {
	const patchUpdates = listen<void>(
		"patcher.patch_updated",
		{ type: "None" },
		() => patcher.get_patcher_state().then(setter),
	);
	const libraryUpdates = listen<void>(
		"patcher.new_fixture",
		{ type: "None" },
		() => patcher.get_patcher_state().then(setter),
	);
	const visualizationUpdates = listen<[string, VisualizationInfo]>(
		"patcher.visualization_updated",
		{ type: "None" },
		(event) => {
			const existingState = mutableGetter();
			if (existingState) {
				existingState.fixtures[event.data[0]].visualization_info = event.data[1];
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
