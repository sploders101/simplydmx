import { Gradient } from "fabric";

/**
 * Creates a fill gradient for a light's visualization
 *
 * Intensity is 0-1 floating-point.
 *
 * Red, green, and blue are 0-255.
 */
export function createGradient(intensity: number, red: number, green: number, blue: number) {
	const color = `rgb(${red}, ${green}, ${blue})`;
	return new Gradient({
		type: "radial",
		gradientUnits: "pixels",
		gradientTransform: [15, 0, 0, 15, 15, 15],
		coords: { r1: 0, r2: 1, x1: 0, x2: 0, y1: 0, y2: 0 },
		colorStops: [
			{ offset: 0, color, opacity: 1 } as any,
			{ offset: 0.5, color, opacity: 1 * intensity } as any,
			{ offset: 0.9, color, opacity: 0 } as any,
		],
	});
}
