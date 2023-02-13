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
