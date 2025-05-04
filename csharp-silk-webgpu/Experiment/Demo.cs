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
	private readonly Buffer<Vertex> vertexBuffer;
	private readonly Buffer<UInt16> indexBuffer;

	public Demo(IWindowState windowState)
	{
		this.windowState = windowState;
		this.videoDriver = (WebGPUVideoDriver)windowState.VideoDriver;

		unsafe
		{
			pipeline = new Pipeline(
				videoDriver,
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
			vertexBuffer = new(
				videoDriver,
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
						new(0.5f, 0.5f),
						System.Drawing.Color.Blue.ToVector()
					),
					new(
						new(-0.5f, 0.5f),
						System.Drawing.Color.Purple.ToVector()
					),
				],
				BufferUsage.Vertex
			);
			indexBuffer = new(
				videoDriver,
				[
					0,1,2,
					2,3,0,
				],
				BufferUsage.Index
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
				pipeline.DrawBuffers(renderPass.RenderPassEncoder, vertexBuffer, indexBuffer, 0, (uint)indexBuffer.Length);
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
	private readonly WebGPUVideoDriver videoDriver;
	private readonly ShaderModule* shaderModule;
	private readonly RenderPipeline* renderPipeline;

	public Pipeline(WebGPUVideoDriver videoDriver, PipelineDescription pipelineDescription)
	{
		this.videoDriver = videoDriver;

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
		shaderModule = videoDriver.WebGPU.DeviceCreateShaderModule(videoDriver.Device, ref descriptor);
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
					Format = videoDriver.SurfaceTextureFormat,
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
		renderPipeline = videoDriver.WebGPU.DeviceCreateRenderPipeline(videoDriver.Device, ref renderPipelineDescriptor);
		Marshal.FreeHGlobal(vertexEntryPointPtr);
		Marshal.FreeHGlobal(fragmentEntryPointPtr);
		Console.WriteLine("created render pipeline");
	}

	public void Dispose()
	{
		videoDriver.WebGPU.RenderPipelineRelease(renderPipeline);
		videoDriver.WebGPU.ShaderModuleRelease(shaderModule);
	}

	public void DrawBuffer<T>(RenderPassEncoder* renderPassEncoder, Buffer<T> vertexBuffer, uint index, uint length) where T : unmanaged
	{
		videoDriver.WebGPU.RenderPassEncoderSetPipeline(renderPassEncoder, renderPipeline);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, (ulong)(index * sizeof(T)), (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDraw(renderPassEncoder, length, 1, 0, 0);
	}

	public void DrawBuffers<T>(RenderPassEncoder* renderPassEncoder, Buffer<T> vertexBuffer, Buffer<UInt16> indexBuffer, uint index, uint length) where T : unmanaged
	{
		videoDriver.WebGPU.RenderPassEncoderSetPipeline(renderPassEncoder, renderPipeline);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, 0, (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderSetIndexBuffer(renderPassEncoder, indexBuffer.Instance, IndexFormat.Uint16, 0, (ulong)indexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDrawIndexed(renderPassEncoder, length, 1, index, 0, 0);
	}
}

unsafe class Buffer<T> : IDisposable where T : unmanaged
{
	private readonly WebGPUVideoDriver videoDriver;
	private readonly int length;
	private readonly int lengthInBytes;
	private readonly int paddedLength;
	private readonly int paddedLengthInBytes;
	private readonly Silk.NET.WebGPU.Buffer* buffer;

	public Buffer(WebGPUVideoDriver videoDriver, ReadOnlySpan<T> data, BufferUsage usage)
	{
		this.videoDriver = videoDriver;
		this.length = data.Length;
		this.lengthInBytes = this.length * sizeof(T);

		// if you don't have the input data size be a multiple of COPY_BUFFER_ALIGNMENT you get
		// Copy size 6 does not respect `COPY_BUFFER_ALIGNMENT`
		// or whatever the input byte size is
		var lengthInBytes = data.Length * sizeof(T);
		if (lengthInBytes % 4 != 0)
		{
			while (lengthInBytes % 4 != 0)
			{
				lengthInBytes += sizeof(T);
			}
			var desiredLength = lengthInBytes / sizeof(T);
			var newData = new T[desiredLength];
			data.CopyTo(newData);
			data = newData;
		}
		this.paddedLength = lengthInBytes / sizeof(T);
		this.paddedLengthInBytes = lengthInBytes;

		var descriptor = new BufferDescriptor()
		{
			MappedAtCreation = false,
			Size = (ulong)paddedLengthInBytes,
			Usage = BufferUsage.CopyDst | usage,
		};
		buffer = videoDriver.WebGPU.DeviceCreateBuffer(videoDriver.Device, ref descriptor);

		videoDriver.WebGPU.QueueWriteBuffer<T>(videoDriver.Queue, buffer, 0, data, (nuint)paddedLengthInBytes);
	}

	public void Dispose()
	{
		videoDriver.WebGPU.BufferRelease(buffer);
	}

	public int Length => length;

	public int SizeInBytes => sizeof(T) * length;

	public Silk.NET.WebGPU.Buffer* Instance => buffer;
}

unsafe class RenderPass
{
	public required RenderPassEncoder* RenderPassEncoder { get; init; }
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