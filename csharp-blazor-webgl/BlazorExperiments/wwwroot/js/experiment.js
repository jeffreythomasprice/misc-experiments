/**
 * @param {HTMLCanvasElement} canvas
 */
export function init(dotNetObj, canvas) {
	const context = canvas.getContext("webgl2", { powerPreference: "high-performance" });

	const resize = () => {
		const width = window.innerWidth;
		const height = window.innerHeight;
		canvas.width = width;
		canvas.height = height;
		dotNetObj.invokeMethod("Resize", width, height);
	};
	window.addEventListener("resize", () => resize());
	resize();

	const anim = time => {
		dotNetObj.invokeMethod("Anim", time);
		requestAnimationFrame(time => anim(time));
	};
	requestAnimationFrame(time => anim(time));

	context.bufferData_float32 = (type, bytes, usage) => {
		context.bufferData(type, new Float32Array(bytes), usage);
	};

	return context;
}
