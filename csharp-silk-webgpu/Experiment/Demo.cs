using System;
using System.ComponentModel.DataAnnotations;
using System.Reflection;
using System.Runtime.InteropServices;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

[AttributeUsage(AttributeTargets.Field)]
class VertexAttribute : Attribute
{
	public required VertexFormat Format { get; init; }
	public required uint ShaderLocation { get; init; }
}

struct Vertex
{
	[VertexAttribute(Format = VertexFormat.Float32x2, ShaderLocation = 0)]
	public readonly Vector2D<float> Position;
	[VertexAttribute(Format = VertexFormat.Float32x4, ShaderLocation = 1)]
	public readonly Vector4D<float> Color;

	public Vertex(Vector2D<float> position, Vector4D<float> color)
	{
		this.Position = position;
		this.Color = color;
	}
}

class Demo : IAppState
{
	private readonly IWindowState windowState;
	private readonly WebGPUVideoDriver videoDriver;

	private readonly Pipeline pipeline;
	private readonly Buffer<Vertex> buffer;

	public Demo(IWindowState windowState)
	{
		this.windowState = windowState;
		this.videoDriver = (WebGPUVideoDriver)windowState.VideoDriver;

		unsafe
		{
			pipeline = new Pipeline(
				videoDriver.WebGPU,
				videoDriver.Device,
				videoDriver.SurfaceTextureFormat,
				new()
				{
					ShaderDescription = new()
					{
						Source = App.EmbeddedFileAsString("Experiment.Assets.Shaders.shader.wgsl"),
						VertexEntryPoint = "vs_main",
						FragmentEntryPoint = "fs_main",
					},
					VertexBufferDescription = VertexBufferDescription<Vertex>.Create()
				}
			);
			buffer = new(
				videoDriver.WebGPU,
				videoDriver.Device,
				videoDriver.Queue,
				[
					new(
						new(-0.5f, -0.5f),
						System.Drawing.Color.Red.ToVector()
					),
					new(
						new(0.5f, -0.5f),
						System.Drawing.Color.Green.ToVector()
					),
					new(
						new(0.0f, 0.5f),
						System.Drawing.Color.Blue.ToVector()
					),
				]
			);
		}
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
			videoDriver.RenderPass((renderPass) =>
			{
				pipeline.Render(renderPass.RenderPassEncoder, buffer);
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

class ShaderDescription
{
	public required string Source { get; init; }
	public required string VertexEntryPoint { get; init; }
	public required string FragmentEntryPoint { get; init; }
}

class VertexBufferDescription
{
	public required int Stride { get; init; }
	public required Silk.NET.WebGPU.VertexAttribute[] Attributes { get; init; }
}

class VertexBufferDescription<T> : VertexBufferDescription where T : unmanaged
{
	unsafe static public VertexBufferDescription<T> Create()
	{
		var attributes = new List<Silk.NET.WebGPU.VertexAttribute>();
		foreach (var field in typeof(T).GetFields())
		{
			var attr = field.GetCustomAttribute<VertexAttribute>();
			if (attr != null)
			{
				var offset = Marshal.OffsetOf<T>(field.Name);
				Console.WriteLine($"vertex attribute {field}, format={attr.Format}, offset={offset}, shaderLocation={attr.ShaderLocation}");
				attributes.Add(new Silk.NET.WebGPU.VertexAttribute()
				{
					Format = attr.Format,
					Offset = (ulong)offset,
					ShaderLocation = attr.ShaderLocation,
				});
			}
		}
		return new()
		{
			Stride = sizeof(T),
			Attributes = attributes.ToArray(),
		};
	}
}

class PipelineDescription
{
	public required ShaderDescription ShaderDescription { get; init; }
	public required VertexBufferDescription VertexBufferDescription { get; init; }
}

unsafe class Pipeline : IDisposable
{
	private readonly WebGPU webGPU;
	private readonly Device* device;
	private readonly ShaderModule* shaderModule;
	private readonly RenderPipeline* renderPipeline;

	public Pipeline(WebGPU webGPU, Device* device, TextureFormat surfaceTextureFormat, PipelineDescription pipelineDescription)
	{
		this.webGPU = webGPU;
		this.device = device;

		var shaderSourcePtr = Marshal.StringToHGlobalAnsi(pipelineDescription.ShaderDescription.Source);
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

		var vertexEntryPointPtr = Marshal.StringToHGlobalAnsi(pipelineDescription.ShaderDescription.VertexEntryPoint);
		var vertexAttributes = stackalloc Silk.NET.WebGPU.VertexAttribute[pipelineDescription.VertexBufferDescription.Attributes.Length];
		for (var i = 0; i < pipelineDescription.VertexBufferDescription.Attributes.Length; i++)
		{
			vertexAttributes[i] = pipelineDescription.VertexBufferDescription.Attributes[i];
		}
		var vertexBufferLayout = new VertexBufferLayout()
		{
			StepMode = VertexStepMode.Vertex,
			ArrayStride = (ulong)pipelineDescription.VertexBufferDescription.Stride,
			AttributeCount = (nuint)pipelineDescription.VertexBufferDescription.Attributes.Length,
			Attributes = vertexAttributes,
		};
		var vertexState = new VertexState()
		{
			Module = shaderModule,
			EntryPoint = (byte*)vertexEntryPointPtr,
			Buffers = &vertexBufferLayout,
			BufferCount = 1,
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
					Format = surfaceTextureFormat,
					Blend = blendState,
				},
			};
		var fragmentEntryPointPtr = Marshal.StringToHGlobalAnsi(pipelineDescription.ShaderDescription.FragmentEntryPoint);
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

	public void Render<T>(RenderPassEncoder* renderPassEncoder, Buffer<T> buffer) where T : unmanaged
	{
		webGPU.RenderPassEncoderSetPipeline(renderPassEncoder, renderPipeline);
		webGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, buffer.Instance, 0, (ulong)buffer.SizeInBytes);
		webGPU.RenderPassEncoderDraw(renderPassEncoder, (uint)buffer.Length, 1, 0, 0);
	}
}

unsafe class Buffer<T> : IDisposable where T : unmanaged
{
	private WebGPU webGPU;
	private int length;
	private Silk.NET.WebGPU.Buffer* buffer;

	public Buffer(WebGPU webGPU, Device* device, Queue* queue, ReadOnlySpan<T> data)
	{
		this.webGPU = webGPU;
		this.length = data.Length;
		var descriptor = new BufferDescriptor()
		{
			MappedAtCreation = false,
			Size = (ulong)SizeInBytes,
			Usage = BufferUsage.CopyDst | BufferUsage.Vertex,
		};
		buffer = webGPU.DeviceCreateBuffer(device, ref descriptor);
		webGPU.QueueWriteBuffer<T>(queue, buffer, 0, data, (nuint)SizeInBytes);
	}

	public void Dispose()
	{
		webGPU.BufferRelease(buffer);
	}

	public int Length => length;

	public int SizeInBytes => sizeof(T) * length;

	public Silk.NET.WebGPU.Buffer* Instance => buffer;
}

unsafe class RenderPass
{
	public RenderPassEncoder* RenderPassEncoder { get; init; }
}

static class WebGPUExtensions
{
	public unsafe static void RenderPass(this WebGPUVideoDriver videoDriver, Action<RenderPass> callback)
	{
		var webGPU = videoDriver.WebGPU;
		var surface = videoDriver.Surface;
		var device = videoDriver.Device;
		var queue = videoDriver.Queue;

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

	public static Vector4D<float> ToVector(this System.Drawing.Color c)
	{
		return new(
			c.R / 255.0f,
			c.G / 255.0f,
			c.B / 255.0f,
			c.A / 255.0f
		);
	}
}