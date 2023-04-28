import { exhaustiveMatch, FixtureInfo, FixtureMixerOutput } from "@/scripts/api/ipc";

/**
 * Converts 8-bit cmyk values to rgb values
 */
export function cmyk2rgb(
	cyan: number,
	magenta: number,
	yellow: number,
	black: number,
): { red: number, green: number, blue: number } {
	return {
		red: 255 * (1 - cyan) * (1 - black),
		green: 255 * (1 - magenta) * (1 - black),
		blue: 255 * (1 - yellow) * (1 - black),
	};
}

/**
 * Normalizes a channel into 8-bit precision
 */
export function normalizeChannel(
	profile: FixtureInfo,
	fixtureData: FixtureMixerOutput,
	channel: string,
) {
	const channelValue = fixtureData[channel];
	const channelInfo = profile.channels[channel];
	return exhaustiveMatch(channelInfo.size, {
		U8: () => channelValue,
		U16: () => Math.floor(channelValue / 257), // Maps 65535 to 255
	});
}
