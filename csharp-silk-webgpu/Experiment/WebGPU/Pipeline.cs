namespace Experiment.WebGPU;

using System.Reflection;
using System.Runtime.InteropServices;
using Silk.NET.WebGPU;

public class ShaderDescription
{
	public required string Source { get; init; }
	public required string VertexEntryPoint { get; init; }
	public required string FragmentEntryPoint { get; init; }
}

public class VertexBufferDescription
{
	public required int Stride { get; init; }
	public required Silk.NET.WebGPU.VertexAttribute[] Attributes { get; init; }
}

public class VertexBufferDescription<T> : VertexBufferDescription where T : unmanaged
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

public class PipelineDescription
{
	public required ShaderDescription ShaderDescription { get; init; }
	public required VertexBufferDescription VertexBufferDescription { get; init; }
}

public unsafe class Pipeline : IDisposable
{
	private readonly VideoDriver videoDriver;
	private readonly ShaderModule* shaderModule;
	private readonly RenderPipeline* renderPipeline;

	public Pipeline(VideoDriver videoDriver, PipelineDescription pipelineDescription)
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
