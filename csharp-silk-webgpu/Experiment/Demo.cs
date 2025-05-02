using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

class Demo : IAppState
{
	private readonly IWindowState windowState;

	public Demo(IWindowState windowState)
	{
		this.windowState = windowState;
	}

	public void Load() { }

	public void Unload() { }

	public void Resize(Vector2D<int> size) { }

	public void Render()
	{
		unsafe
		{
			var webGPU = windowState.WebGPU;
			var surface = windowState.Surface;
			var device = windowState.Device;

			var queue = webGPU.DeviceGetQueue(device);

			var commandEncoder = webGPU.DeviceCreateCommandEncoder(device, null);

			SurfaceTexture surfaceTexture = default;
			webGPU.SurfaceGetCurrentTexture(surface, ref surfaceTexture);

			var surfaceTextureView = webGPU.TextureCreateView(surfaceTexture.Texture, null);

			var clearColor = System.Drawing.Color.CornflowerBlue;
			var colorAttachments = stackalloc RenderPassColorAttachment[] {
				new() {
					View = surfaceTextureView,
					LoadOp = LoadOp.Clear,
					ClearValue = new (clearColor.R/255.0, clearColor.G/255.0, clearColor.B/255.0, 1.0),
					StoreOp = StoreOp.Store,
				}
			};
			var renderPassDescriptor = new RenderPassDescriptor()
			{
				ColorAttachmentCount = 1,
				ColorAttachments = colorAttachments,
			};
			var renderPassEncoder = webGPU.CommandEncoderBeginRenderPass(commandEncoder, ref renderPassDescriptor);

			// TODO render stuff

			webGPU.RenderPassEncoderEnd(renderPassEncoder);

			var commandBuffer = webGPU.CommandEncoderFinish(commandEncoder, null);
			webGPU.QueueSubmit(queue, 1, &commandBuffer);

			webGPU.SurfacePresent(surface);

			webGPU.TextureViewRelease(surfaceTextureView);
			webGPU.TextureRelease(surfaceTexture.Texture);
			webGPU.RenderPassEncoderRelease(renderPassEncoder);
			webGPU.CommandBufferRelease(commandBuffer);
			webGPU.CommandEncoderRelease(commandEncoder);
		}
	}

	public AppStateTransition? Update(TimeSpan delta)
	{
		return null;
	}

	public AppStateTransition? KeyDown(Key key)
	{
		return null;
	}

	public AppStateTransition? KeyUp(Key key)
	{
		if (key == Key.Escape)
		{
			return AppStateTransition.Exit;
		}
		return null;
	}
}