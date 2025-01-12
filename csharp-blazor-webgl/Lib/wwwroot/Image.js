export function init() {
	const fixImage = (image) => {
		image.getWidth = () => image.width;
		image.getHeight = () => image.height;
		return image;
	};

	return {
		loadImageUrl: (url) => {
			const result = fixImage(new Image());
			return new Promise((resolve, reject) => {
				result.addEventListener("load", () => resolve(result));
				result.addEventListener("error", (err) => reject(err));
				result.src = url;
			});
		},
	};
}