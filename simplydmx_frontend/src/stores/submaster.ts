import { computed, ref, reactive, onMounted, onUnmounted, watch } from "vue";
import * as ipc from "@/scripts/api/ipc";
import { listen } from "@/scripts/api/ipc";

interface SubmasterInfo {
	data: ipc.SubmasterData | null,
	listeners: number,
	unlistener: Promise<() => Promise<void>>,
}
const submasters = reactive(new Map<ipc.Uuid, SubmasterInfo>());

/**
 * Merges the submaster data from `src` into `dest`, mutating `dest`.
 *
 * Objects from `src` may end up in dest, meaning `src` could be mutated later.
 */
function mergeSubmasterData(src: ipc.SubmasterData, dest: ipc.SubmasterData): void {
	Object.entries(src).forEach(([fixtureId, fixtureValues]) => {
		let destFixtureValues = dest[fixtureId];
		if (destFixtureValues) {
			Object.entries(fixtureValues).forEach(([channelId, channelValue]) => {
				destFixtureValues[channelId] = channelValue;
			});
		} else {
			dest[fixtureId] = fixtureValues;
		}
	});
}

function setup(submasterId: ipc.Uuid) {
	const state = submasters.get(submasterId) || (() => {
		const state: SubmasterInfo = reactive({
			data: null,
			listeners: 0,
			unlistener: listen<ipc.SubmasterData>(
				"mixer.submaster_updated",
				{ type: "Uuid", data: submasterId },
				// { type: "None" },
				(delta) => {
					if (state.data) {
						mergeSubmasterData(delta.data, state.data);
					} else {
						state.data = delta.data;
					}
				},
			),
		});
		submasters.set(submasterId, state);
		ipc.mixer.get_layer_contents(submasterId)
			.then((data) => {
				if (data && state.data) {
					mergeSubmasterData(data.values, state.data);
				} else {
					state.data = data?.values || null;
				}
			});
		return state;
	})();
	state.listeners++;
}

function teardown(submasterId: ipc.Uuid) {
	const state = submasters.get(submasterId)!;
	state.listeners--;

	if (state.listeners === 0) {
		submasters.delete(submasterId);
		state.unlistener.then((unlistener) => unlistener());
	}
}

/**
 * Creates a reactive binding to a submaster by ID, yielding its values,
 * keeping synchronization with the server.
 */
export function useSubmasterData(submasterId: () => ipc.Uuid | null) {
	const mounted = ref(false);
	onMounted(() => mounted.value = true);
	onUnmounted(() => {
		const submasterIdValue = submasterId();
		if (submasterIdValue) teardown(submasterIdValue);
	});
	watch([submasterId, mounted], ([newSubmasterId, newMounted], [oldSubmasterId, oldMounted]) => {
		if (newSubmasterId === null && typeof oldSubmasterId === "string" && oldMounted && newMounted) teardown(oldSubmasterId);
		if (newMounted) {
			if (!oldMounted) {
				if (newSubmasterId) setup(newSubmasterId);
			} else if (newSubmasterId !== oldSubmasterId) {
				if (oldSubmasterId) teardown(oldSubmasterId);
				if (newSubmasterId) setup(newSubmasterId);
			}
		} else if(oldMounted) {
			if (oldSubmasterId) teardown(oldSubmasterId);
		}
	}, { immediate: true });
	return computed(() => {
		const submasterIdValue = submasterId();
		if (submasterIdValue === null) return null;
		return submasters.get(submasterIdValue)?.data || null;
	});
}
