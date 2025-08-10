import sdl from '@kmamal/sdl'
import gpu from '@kmamal/gpu'

(async () => {
	// Setup
	const window = sdl.video.createWindow({ title: "WebGPU", webgpu: true });
	window.on("close", () => {
		process.exit(0);
	});

	const instance = gpu.create([]);
	const adapter = await instance.requestAdapter();
	if (!adapter) {
		throw new Error("Failed to get GPU adapter");
	}
	const device = await adapter.requestDevice();
	const renderer = gpu.renderGPUDeviceToWindow({ device, window });

	// Clear screen to red
	const commandEncoder = device.createCommandEncoder();
	const renderPass = commandEncoder.beginRenderPass({
		colorAttachments: [
			{
				view: renderer.getCurrentTextureView(),
				clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
				loadOp: 'clear',
				storeOp: 'store',
			},
		],
	});
	renderPass.end();
	device.queue.submit([commandEncoder.finish()]);

	// Render to window
	renderer.swap();
})()
	.catch(err => {
		console.error("oops", err);
		process.exit(1);
	});