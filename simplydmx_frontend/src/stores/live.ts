import { ref, onMounted, onUnmounted } from "vue";
import * as ipc from "@/scripts/api/ipc";
import { listen } from "@/scripts/api/ipc";

const liveMix = ref<null | ipc.FullMixerOutput>(null);
let unlisten: (Promise<() => Promise<void>>) | null = null;
let listeners = 0;

export function useLiveMixState() {
	onMounted(() => {
		if (listeners === 0) {
			unlisten = listen<ipc.FullMixerOutput>(
				"mixer.final_output",
				{ type: "None" },
				(event) => liveMix.value = event.data,
			);
			// This runs after we've subscribed, but doesn't interfere with the promise result
			unlisten.then(ipc.mixer.request_blend);
		}
		listeners += 1;
	});
	onUnmounted(() => {
		listeners -= 1;
		if (listeners === 0) {
			let unlistener = unlisten;
			unlisten = null;
			liveMix.value = null;
			unlistener?.then((unlistener) => unlistener());
		}
	});

	return liveMix;
}
