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
			var webGPUState = windowState.WebGPUState;

			var webGPU = webGPUState.WebGPU;
			var surface = webGPUState.Surface;
			var device = webGPUState.Device;

			var queue = webGPU.DeviceGetQueue(device);

			var commandEncoder = webGPU.DeviceCreateCommandEncoder(device, null);

			SurfaceTexture surfaceTexture = default;
			webGPU.SurfaceGetCurrentTexture(surface, ref surfaceTexture);

			var surfaceTextureView = webGPU.TextureCreateView(surfaceTexture.Texture, null);

			var colorAttachments = stackalloc RenderPassColorAttachment[] {
				new() {
					View = surfaceTextureView,
					LoadOp = LoadOp.Clear,
					ClearValue = System.Drawing.Color.CornflowerBlue.ToWebGPU(),
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

public static class ColorExtensions
{
	public static Silk.NET.WebGPU.Color ToWebGPU(this System.Drawing.Color c)
	{
		return new(
			c.R / 255.0,
			c.G / 255.0,
			c.B / 255.0,
			c.A / 255.0
		);
	}
}