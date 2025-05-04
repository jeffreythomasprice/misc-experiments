namespace Experiment.WebGPU;

using System.Reflection;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

public class ShaderDescription
{
	public required string Source { get; init; }
	public required string VertexEntryPoint { get; init; }
	public required string FragmentEntryPoint { get; init; }
}

record class VertexDescription(int Stride, List<Silk.NET.WebGPU.VertexAttribute> Attributes);

public unsafe class Pipeline<T> : IDisposable where T : unmanaged
{
	private readonly VideoDriver videoDriver;
	private readonly Buffer<Matrix4X4<float>> projectionMatrixBuffer;
	private readonly ShaderModule* shaderModule;
	private readonly RenderPipeline* renderPipeline;
	private readonly BindGroup* bindGroup;

	public Pipeline(VideoDriver videoDriver, ShaderDescription shaderDescription)
	{
		this.videoDriver = videoDriver;

		projectionMatrixBuffer = new Buffer<Matrix4X4<float>>(videoDriver, [Matrix4X4<float>.Identity], BufferUsage.Uniform);

		shaderModule = CreateShader(videoDriver, shaderDescription);

		var bindGroupLayoutEntries = stackalloc BindGroupLayoutEntry[]
		{
			new()
			{
				Binding = 0,
				Visibility = ShaderStage.Vertex,
				Buffer = new()
				{
					Type = BufferBindingType.Uniform,
				},
			},
		};
		var bindGroupLayoutDescriptor = new BindGroupLayoutDescriptor()
		{
			Entries = bindGroupLayoutEntries,
			EntryCount = 1,
		};
		var bindGroupLayout = videoDriver.WebGPU.DeviceCreateBindGroupLayout(videoDriver.Device, ref bindGroupLayoutDescriptor);
		var bindGroupLayouts = stackalloc BindGroupLayout*[]
		{
			bindGroupLayout,
		};
		var pipelineLayoutDescriptor = new PipelineLayoutDescriptor()
		{
			BindGroupLayouts = bindGroupLayouts,
			BindGroupLayoutCount = 1,
		};
		var pipelineLayout = videoDriver.WebGPU.DeviceCreatePipelineLayout(videoDriver.Device, ref pipelineLayoutDescriptor);

		var vertexDescription = CreateVertexDescription();
		var vertexEntryPointPtr = Marshal.StringToHGlobalAnsi(shaderDescription.VertexEntryPoint);
		var vertexAttributes = stackalloc Silk.NET.WebGPU.VertexAttribute[vertexDescription.Attributes.Count];
		for (var i = 0; i < vertexDescription.Attributes.Count; i++)
		{
			vertexAttributes[i] = vertexDescription.Attributes[i];
		}
		var vertexBufferLayout = new VertexBufferLayout()
		{
			StepMode = VertexStepMode.Vertex,
			ArrayStride = (ulong)vertexDescription.Stride,
			AttributeCount = (nuint)vertexDescription.Attributes.Count,
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
		var fragmentEntryPointPtr = Marshal.StringToHGlobalAnsi(shaderDescription.FragmentEntryPoint);
		var fragmentState = new FragmentState()
		{
			Module = shaderModule,
			EntryPoint = (byte*)fragmentEntryPointPtr,
			Targets = colorTargetState,
			TargetCount = 1,
		};

		var renderPipelineDescriptor = new RenderPipelineDescriptor()
		{
			Layout = pipelineLayout,
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

		var bindGroupEntries = stackalloc BindGroupEntry[]
		{
			new()
			{
				Binding = 0,
				Buffer = projectionMatrixBuffer.Instance,
				Size = (ulong)projectionMatrixBuffer.SizeInBytes,
			},
		};
		var bindGroupDescriptor = new BindGroupDescriptor()
		{
			Layout = bindGroupLayout,
			Entries = bindGroupEntries,
			EntryCount = 1,
		};
		bindGroup = videoDriver.WebGPU.DeviceCreateBindGroup(videoDriver.Device, ref bindGroupDescriptor);
	}

	public void Dispose()
	{
		projectionMatrixBuffer.Dispose();
		videoDriver.WebGPU.RenderPipelineRelease(renderPipeline);
		// TODO shouldn't need a ref to shader because pipeline should retain it?
		videoDriver.WebGPU.ShaderModuleRelease(shaderModule);
		videoDriver.WebGPU.BindGroupRelease(bindGroup);
	}

	public void QueueWriteProjectionMatrix(Matrix4X4<float> m)
	{
		projectionMatrixBuffer.QueueWrite([m], 0);
	}

	public void DrawBuffer(RenderPassEncoder* renderPassEncoder, Buffer<T> vertexBuffer, uint index, uint length)
	{
		DrawCommon(renderPassEncoder);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, (ulong)(index * vertexBuffer.Stride), (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDraw(renderPassEncoder, length, 1, 0, 0);
	}

	public void DrawBuffers(RenderPassEncoder* renderPassEncoder, Buffer<T> vertexBuffer, Buffer<UInt16> indexBuffer, uint index, uint length)
	{
		DrawCommon(renderPassEncoder);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, 0, (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderSetIndexBuffer(renderPassEncoder, indexBuffer.Instance, IndexFormat.Uint16, 0, (ulong)indexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDrawIndexed(renderPassEncoder, length, 1, index, 0, 0);
	}

	private void DrawCommon(RenderPassEncoder* renderPassEncoder)
	{
		videoDriver.WebGPU.RenderPassEncoderSetPipeline(renderPassEncoder, renderPipeline);
		videoDriver.WebGPU.RenderPassEncoderSetBindGroup(renderPassEncoder, 0, bindGroup, 0, null);
	}

	private static ShaderModule* CreateShader(VideoDriver videoDriver, ShaderDescription shaderDescription)
	{
		var sourcePtr = Marshal.StringToHGlobalAnsi(shaderDescription.Source);
		var shaderModuleWGSLDescriptor = new ShaderModuleWGSLDescriptor()
		{
			Code = (byte*)sourcePtr,
			Chain = {
					SType = SType.ShaderModuleWgslDescriptor,
				},
		};
		var descriptor = new ShaderModuleDescriptor()
		{
			NextInChain = (ChainedStruct*)&shaderModuleWGSLDescriptor,
		};
		var result = videoDriver.WebGPU.DeviceCreateShaderModule(videoDriver.Device, ref descriptor);
		Marshal.FreeHGlobal(sourcePtr);
		Console.WriteLine("created shader");
		return result;
	}

	private static VertexDescription CreateVertexDescription()
	{
		var stride = Unsafe.SizeOf<T>();
		Console.WriteLine($"vertex stride: {stride}");
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
		return new(stride, attributes);
	}
}
