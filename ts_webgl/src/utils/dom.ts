import { Size2 } from "../geometry";

export function loadImageFromURL(url: URL): Promise<HTMLImageElement> {
	return new Promise((resolve, reject) => {
		const result = document.createElement("img");
		result.addEventListener("load", () => {
			resolve(result);
		});
		result.addEventListener("error", () => {
			reject(new Error(`failed to load from ${url}`));
		});
		result.src = url.toString();
	});
}

export async function loadFontFromURL(family: string, url: URL): Promise<FontFace> {
	const response = await fetch(url);
	const responseBody = await response.arrayBuffer();
	return await new FontFace(family, responseBody).load();
}

export function createOffscreenCanvasAndContext(size: Size2): [OffscreenCanvas, OffscreenCanvasRenderingContext2D] {
	const canvas = new OffscreenCanvas(size.width, size.height);
	const context = canvas.getContext("2d");
	if (!context) {
		throw new Error("failed to make offscreen rendering context");
	}
	return [canvas, context];
}
