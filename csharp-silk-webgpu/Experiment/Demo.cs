using System.Runtime.InteropServices;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

class Demo : IAppState
{
	private readonly IWindowState windowState;

	private readonly Pipeline pipeline;

	public Demo(IWindowState windowState)
	{
		this.windowState = windowState;

		pipeline = new Pipeline(windowState.WebGPUState, App.EmbeddedFileAsString("Experiment.Assets.Shaders.shader.wgsl"));
	}

	public void Load() { }

	public void Unload()
	{
		pipeline.Dispose();
	}

	public void Resize(Vector2D<int> size) { }

	public void Render()
	{
		unsafe
		{
			windowState.WebGPUState.RenderPass((renderPass) =>
			{
				pipeline.Render(renderPass.RenderPassEncoder);
			});
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

unsafe class Pipeline : IDisposable
{
	private readonly WebGPU webGPU;
	private readonly Device* device;
	private readonly ShaderModule* shaderModule;
	private readonly RenderPipeline* renderPipeline;

	public Pipeline(WebGPUState webGPUState, string shaderSource)
	{
		this.webGPU = webGPUState.WebGPU;
		this.device = webGPUState.Device;

		var shaderSourcePtr = Marshal.StringToHGlobalAnsi(shaderSource);
		var shaderModuleWGSLDescriptor = new ShaderModuleWGSLDescriptor()
		{
			Code = (byte*)shaderSourcePtr,
			Chain = {
				SType = SType.ShaderModuleWgslDescriptor,
			},
		};
		var descriptor = new ShaderModuleDescriptor()
		{
			NextInChain = (ChainedStruct*)&shaderModuleWGSLDescriptor,
		};
		shaderModule = webGPU.DeviceCreateShaderModule(device, ref descriptor);
		Marshal.FreeHGlobal(shaderSourcePtr);
		Console.WriteLine("created shader");

		var vertexEntryPointPtr = Marshal.StringToHGlobalAnsi("vs_main");
		var vertexState = new VertexState()
		{
			Module = shaderModule,
			EntryPoint = (byte*)vertexEntryPointPtr,
		};
		var blendState = stackalloc BlendState[] {
			new()
			{
				Color = new()
				{
					SrcFactor = BlendFactor.One,
					DstFactor = BlendFactor.OneMinusSrcAlpha,
					Operation = BlendOperation.Add,
				},
				Alpha = new()
				{
					SrcFactor = BlendFactor.One,
					DstFactor = BlendFactor.OneMinusSrcAlpha,
					Operation = BlendOperation.Add,
				},
			},
		};
		var colorTargetState = stackalloc ColorTargetState[]
		{
			new()
			{
				WriteMask = ColorWriteMask.All,
				// TODO use surface texture format?
				Format = TextureFormat.Bgra8Unorm,
				Blend = blendState,
			},
		};
		var fragmentEntryPointPtr = Marshal.StringToHGlobalAnsi("fs_main");
		var fragmentState = new FragmentState()
		{
			Module = shaderModule,
			EntryPoint = (byte*)fragmentEntryPointPtr,
			Targets = colorTargetState,
			TargetCount = 1,
		};
		var renderPipelineDescriptor = new RenderPipelineDescriptor()
		{
			Vertex = vertexState,
			Fragment = &fragmentState,
			Multisample = new()
			{
				Mask = 0xffffffff,
				Count = 1,
				AlphaToCoverageEnabled = false,
			},
			Primitive = new()
			{
				CullMode = CullMode.None,
				FrontFace = FrontFace.Ccw,
				Topology = PrimitiveTopology.TriangleList,
			},
		};
		renderPipeline = webGPU.DeviceCreateRenderPipeline(device, ref renderPipelineDescriptor);
		Marshal.FreeHGlobal(vertexEntryPointPtr);
		Marshal.FreeHGlobal(fragmentEntryPointPtr);
		Console.WriteLine("created render pipeline");
	}

	public void Dispose()
	{
		webGPU.RenderPipelineRelease(renderPipeline);
		webGPU.ShaderModuleRelease(shaderModule);
	}

	public void Render(RenderPassEncoder* renderPassEncoder)
	{
		webGPU.RenderPassEncoderSetPipeline(renderPassEncoder, renderPipeline);
		webGPU.RenderPassEncoderDraw(renderPassEncoder, 3, 1, 0, 0);
	}
}

unsafe class RenderPass
{
	public RenderPassEncoder* RenderPassEncoder { get; init; }
}

static class WebGPUExtensions
{
	public unsafe static void RenderPass(this WebGPUState webGPUState, Action<RenderPass> callback)
	{
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

		callback(new()
		{
			RenderPassEncoder = renderPassEncoder,
		});

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

static class ColorExtensions
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