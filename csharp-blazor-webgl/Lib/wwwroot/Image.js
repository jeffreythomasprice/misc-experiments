export function init() {
	const fixImage = (image) => {
		image.getWidth = () => image.width;
		image.getHeight = () => image.height;
		return image;
	};

	return {
		loadImageUrl: (url) => {
			console.log("TODO loadImageUrl", url);
			const result = fixImage(document.createElement("image"));
			return new Promise((resolve, reject) => {
				result.addEventListener("load", () => {
					console.log("TODO loadImageUrl success");
					resolve(result);
				});
				result.addEventListener("error", (err) => {
					console.error("TODO loadImageUrl error", err);
					reject(err);
				});
				result.src = url;
			});
		},
	};
}