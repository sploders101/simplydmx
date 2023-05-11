import { FullMixerOutput, SubmasterData } from "@/scripts/api/ipc/rpc";
/**
 * This interface describes the interface between the visualizer
 * and the visualization & control info (ie. what is shown on the
 * lights, what is shown in the control panel, and what happens
 * when the user makes changes).
 */
export interface VisualizerConnector {
	visualizerData: () => FullMixerOutput | SubmasterData | null,
	setProperties: (data: SubmasterData) => void,
}
