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

	context.getActiveInfoName = (activeInfo) => {
		return activeInfo.name;
	};

	context.getActiveInfoType = (activeInfo) => {
		return activeInfo.type;
	};

	context.getActiveInfoSize = (activeInfo) => {
		return activeInfo.size;
	};

	context.bufferData_uint8 = (target, bytes, usage) => {
		context.bufferData(target, new Uint8Array(bytes), usage);
	};

	context.bufferData_float32 = (target, bytes, usage) => {
		context.bufferData(target, new Float32Array(bytes), usage);
	};

	context.bufferData_float64 = (target, bytes, usage) => {
		context.bufferData(target, new Float64Array(bytes), usage);
	};

	context.bufferSubData_uint8 = (target, offset, data) => {
		context.bufferSubData(target, offset, new Uint8Array(data));
	};

	context.bufferSubData_float32 = (target, offset, data) => {
		context.bufferSubData(target, offset, new Float32Array(data));
	};

	context.bufferSubData_float64 = (target, offset, data) => {
		context.bufferSubData(target, offset, new Float64Array(data));
	};

	return context;
}
