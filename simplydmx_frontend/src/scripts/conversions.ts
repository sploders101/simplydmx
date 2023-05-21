import { AbstractLayerLight, BlenderValue, exhaustiveMatch, FixtureInfo, FixtureMixerOutput } from "@/scripts/api/ipc";

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
 * Converts rgb values to cmyk
 */
export function rgb2cmyk(red: number, green: number, blue: number, scale: number = 255) {
	let cyan = scale - red;
	let magenta = scale - green;
	let yellow = scale - blue;
	let black = 0;

	let minCMY = Math.min(cyan, magenta, yellow);

	cyan = (cyan - minCMY) / (scale - minCMY) ;
	magenta = (magenta - minCMY) / (scale - minCMY) ;
	yellow = (yellow - minCMY) / (scale - minCMY) ;
	black = minCMY;

	return {
		cyan: cyan * scale,
		magenta: magenta * scale,
		yellow: yellow * scale,
		black: black * scale,
	};
}

export function normalizeSubmasterValues(channelValue: number | BlenderValue, defaultValue: number | undefined): number {
	if (typeof channelValue === "number") {
		return channelValue;
	} else {
		return exhaustiveMatch(channelValue, {
			None: () => defaultValue || 0,
			Offset: (offset) => (defaultValue || 0) + offset,
			Static: (value) => value,
		});
	}
}

export function parseColorString(color: string): [number, number, number] {
	return [
		parseInt(color.substring(1, 3), 16),
		parseInt(color.substring(3, 5), 16),
		parseInt(color.substring(5, 7), 16),
	];
}

export function formatColorString(red: number, green: number, blue: number): string {
	return "#" + [red, green, blue].map((value) => {
		const hex = value.toString(16);
		if (hex.length === 1) {
			return "0" + hex;
		} else {
			return hex;
		}
	}).join("");
}

/**
 * Normalizes a channel into 8-bit precision
 */
export function normalizeChannel(
	profile: FixtureInfo,
	fixtureData: FixtureMixerOutput | AbstractLayerLight,
	channel: string,
	desiredScale: "U8" | "U16" | "percentage" = "U8",
): number {
	const channelInfo = profile.channels[channel];
	const channelValue = normalizeSubmasterValues(fixtureData[channel], channelInfo.default);
	switch (desiredScale) {
		case "U8":
			return exhaustiveMatch(channelInfo.size, {
				U8: () => channelValue,
				U16: () => Math.floor(channelValue / 257), // Maps 65535 to 255
			});
		case "U16":
			return exhaustiveMatch(channelInfo.size, {
				U8: () => channelValue * 257, // Maps 255 to 65535
				U16: () => channelValue,
			});
		case "percentage":
			return exhaustiveMatch(channelInfo.size, {
				U8: () => channelValue / 255,
				U16: () => channelValue / 65535,
			});
	}
}
