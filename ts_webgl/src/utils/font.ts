import { Aabb2, Size2, Vector2 } from "../geometry";
import { createOffscreenCanvasAndContext } from "./dom";

export interface TextPlacementAlignmentOptions {
	bounds?: Aabb2;
	horizontal?: number | "left" | "center" | "right";
	vertical?: number | "top" | "center" | "bottom";
}

export interface TextPlacement {
	readonly font: string;
	readonly bounds: Aabb2;
	lines: {
		readonly text: string;
		readonly bounds: Aabb2;
		readonly ascent: number;
	}[];
}

export function getTextPlacement(
	font: string,
	text: string,
	alignmentOptions?: TextPlacementAlignmentOptions,
): TextPlacement {
	const destinationBounds = alignmentOptions?.bounds ?? new Aabb2(new Vector2(0, 0), new Size2(0, 0));
	let halign = alignmentOptions?.horizontal ?? 0;
	if (typeof halign === "string") {
		switch (halign) {
			case "left":
				halign = 0;
				break;
			case "center":
				halign = 0.5;
				break;
			case "right":
				halign = 1;
				break;
		}
	}
	let valign = alignmentOptions?.vertical ?? 0;
	if (typeof valign === "string") {
		switch (valign) {
			case "top":
				valign = 0;
				break;
			case "center":
				valign = 0.5;
				break;
			case "bottom":
				valign = 1;
				break;
		}
	}

	const [_canvas, context] = createOffscreenCanvasAndContext(new Size2(0, 0));
	context.font = font;

	const sampleLineMetrics = context.measureText("M");
	const lines = text.split("\n").map((line) => {
		const metrics = context.measureText(line);
		return { line, metrics };
	});
	const { maxAscent, maxDescent } = [sampleLineMetrics, ...lines.map(({ metrics }) => metrics)]
		.reduce(
			({ maxAscent, maxDescent }, metrics) => {
				return {
					maxAscent: Math.max(maxAscent, metrics.actualBoundingBoxAscent),
					maxDescent: Math.max(maxDescent, metrics.actualBoundingBoxDescent),
				};
			},
			{
				maxAscent: 0,
				maxDescent: 0,
			}
		);
	const lineHeight = maxAscent + maxDescent;
	const totalHeight = lineHeight * lines.length;

	const resultLines: TextPlacement["lines"] = [];
	let y = destinationBounds.y + (destinationBounds.height - totalHeight) * valign;
	for (const line of lines) {
		const x = destinationBounds.x + (destinationBounds.width - line.metrics.width) * halign;
		const height = line.metrics.actualBoundingBoxAscent + line.metrics.actualBoundingBoxDescent;
		resultLines.push({
			text: line.line,
			bounds: new Aabb2(new Vector2(x, y), new Size2(line.metrics.width, height)),
			ascent: line.metrics.actualBoundingBoxAscent,
		});
		y += lineHeight;
	}
	return {
		font,
		bounds: Aabb2.fromPoints(resultLines.flatMap(({ bounds }) => [bounds.min, bounds.max])),
		lines: resultLines,
	};
}