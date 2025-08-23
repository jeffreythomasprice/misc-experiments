using Experiment.WebGPU;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.WebGPU;
using Silk.NET.Windowing;

var mainMonitorBounds = Silk.NET.Windowing.Monitor.GetMainMonitor(null).Bounds;
var windowSize = new Vector2D<int>(1024, 768);

var window = Window.Create(new()
{
	IsVisible = true,
	Size = windowSize,
	Title = "Experiment",
	API = GraphicsAPI.None,
	VSync = false,
	Position = new(
		mainMonitorBounds.Origin.X + (mainMonitorBounds.Size.X - windowSize.X) / 2,
		mainMonitorBounds.Origin.Y + (mainMonitorBounds.Size.Y - windowSize.Y) / 2
	),
	WindowState = WindowState.Normal,
	WindowBorder = WindowBorder.Fixed,
	VideoMode = VideoMode.Default,
});

State? state = null;

window.Load += () =>
{
	state = new State(window);
	state.Resize(window.Size);
};

window.Closing += () => { };

window.FramebufferResize += (size) =>
{
	state?.Resize(size);
};

window.Update += (time) => { };

window.Render += (time) =>
{
	state?.Render();
};

window.Initialize();

var inputContext = window.CreateInput();
foreach (var keyboard in inputContext.Keyboards)
{
	keyboard.KeyDown += (kb, key, code) =>
	{
		if (key == Key.Escape)
		{
			window.Close();
		}
	};

	keyboard.KeyUp += (kb, key, code) => { };
}

window.Run();

state?.Dispose();

class State(IWindow window) : IDisposable
{
	private readonly VideoDriver videoDriver = new(window);

	public void Dispose()
	{
		videoDriver.Dispose();
	}

	public void Resize(Vector2D<int> size)
	{
		videoDriver.Resize(size);
	}

	public void Render()
	{
		// TODO make a helper for render pass stuff? put in VideoDriverExtensions?
		unsafe
		{
			var commandEncoder = videoDriver.WebGPU.DeviceCreateCommandEncoder(videoDriver.Device, null);

			SurfaceTexture surfaceTexture = default;
			videoDriver.WebGPU.SurfaceGetCurrentTexture(videoDriver.Surface, ref surfaceTexture);

			var surfaceTextureView = videoDriver.WebGPU.TextureCreateView(surfaceTexture.Texture, null);

			var colorAttachments = stackalloc RenderPassColorAttachment[] {
				new()
				{
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
			var renderPassEncoder = videoDriver.WebGPU.CommandEncoderBeginRenderPass(commandEncoder, ref renderPassDescriptor);

			// TODO do something with renderPassEncoder

			videoDriver.WebGPU.RenderPassEncoderEnd(renderPassEncoder);

			var commandBuffer = videoDriver.WebGPU.CommandEncoderFinish(commandEncoder, null);
			videoDriver.WebGPU.QueueSubmit(videoDriver.Queue, 1, &commandBuffer);

			videoDriver.WebGPU.SurfacePresent(videoDriver.Surface);

			videoDriver.WebGPU.TextureViewRelease(surfaceTextureView);
			videoDriver.WebGPU.TextureRelease(surfaceTexture.Texture);
			videoDriver.WebGPU.RenderPassEncoderRelease(renderPassEncoder);
			videoDriver.WebGPU.CommandBufferRelease(commandBuffer);
			videoDriver.WebGPU.CommandEncoderRelease(commandEncoder);
		}
	}
}