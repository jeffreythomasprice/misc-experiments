/**
 * @param {HTMLCanvasElement} canvas
 */
export function init(dotNetObj, canvas) {
	const context = canvas.getContext("webgl2", { powerPreference: "high-performance" });

	context.resize = () => {
		const width = window.innerWidth;
		const height = window.innerHeight;
		canvas.width = width;
		canvas.height = height;
		dotNetObj.invokeMethod("Resize", width, height);
	};
	window.addEventListener("resize", () => context.resize());

	const anim = time => {
		dotNetObj.invokeMethod("Anim", time);
		requestAnimationFrame(time => anim(time));
	};
	requestAnimationFrame(time => anim(time));

	context.bufferData_uint8 = (type, bytes, usage) => {
		context.bufferData(type, new Uint8Array(bytes), usage);
	};

	context.bufferData_float32 = (type, bytes, usage) => {
		context.bufferData(type, new Float32Array(bytes), usage);
	};

	context.bufferData_float64 = (type, bytes, usage) => {
		context.bufferData(type, new Float64Array(bytes), usage);
	};

	context.bufferSubData_uint8 = (type, offset, data) => {
		context.bufferSubData(type, offset, new Uint8Array(data));
	};

	context.bufferSubData_float32 = (type, offset, data) => {
		context.bufferSubData(type, offset, new Float32Array(data));
	};

	context.bufferSubData_float64 = (type, offset, data) => {
		context.bufferSubData(type, offset, new Float64Array(data));
	};

	return context;
}
