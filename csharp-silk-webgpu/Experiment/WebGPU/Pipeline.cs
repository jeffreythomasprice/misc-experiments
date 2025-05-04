namespace Experiment.WebGPU;

using System.Reflection;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Microsoft.VisualBasic;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

public class ShaderDescription
{
	public required string Source { get; init; }
	public required string VertexEntryPoint { get; init; }
	public required string FragmentEntryPoint { get; init; }
}

record class VertexDescription(int Stride, Silk.NET.WebGPU.VertexAttribute[] Attributes);

public unsafe class Pipeline<T> : IDisposable where T : unmanaged
{
	public unsafe class ModelviewMatrix : IDisposable
	{
		private readonly VideoDriver videoDriver;
		private readonly Buffer<Matrix4X4<float>> buffer;
		private readonly BindGroup* bindGroup;

		internal ModelviewMatrix(VideoDriver videoDriver, Buffer<Matrix4X4<float>> buffer, BindGroup* bindGroup)
		{
			this.videoDriver = videoDriver;
			this.buffer = buffer;
			this.bindGroup = bindGroup;
		}

		public void Dispose()
		{
			buffer.Dispose();
			videoDriver.WebGPU.BindGroupRelease(bindGroup);
		}

		public void QueueWrite(Matrix4X4<float> m)
		{
			buffer.QueueWrite([m], 0);
		}

		internal BindGroup* BindGroup => bindGroup;
	}

	private readonly VideoDriver videoDriver;
	private readonly Buffer<Matrix4X4<float>> projectionMatrixBuffer;
	private readonly BindGroupLayout* modelviewMatrixBindGroupLayout;
	private readonly RenderPipeline* renderPipeline;
	private readonly BindGroup* projectionMatrixBindGroup;

	public Pipeline(VideoDriver videoDriver, ShaderDescription shaderDescription)
	{
		this.videoDriver = videoDriver;

		projectionMatrixBuffer = new Buffer<Matrix4X4<float>>(videoDriver, [Matrix4X4<float>.Identity], BufferUsage.Uniform);

		var shaderModule = CreateShader(videoDriver, shaderDescription);
		var projectionMatrixBindGroupLayout = CreateBindGroupLayout(
			videoDriver,
			[
				new()
				{
					Binding = 0,
					Visibility = ShaderStage.Vertex,
					Buffer = new()
					{
						Type = BufferBindingType.Uniform,
					},
				},
			]
		);
		modelviewMatrixBindGroupLayout = CreateBindGroupLayout(
			videoDriver,
			[
				new()
				{
					Binding = 0,
					Visibility = ShaderStage.Vertex,
					Buffer = new()
					{
						Type = BufferBindingType.Uniform,
					},
				},
			]
		);
		var pipelineLayout = CreatePipelineLayout(videoDriver, [projectionMatrixBindGroupLayout, modelviewMatrixBindGroupLayout]);
		var vertexDescription = CreateVertexDescription();
		renderPipeline = CreateRenderPipeline(videoDriver, shaderModule, shaderDescription.VertexEntryPoint, shaderDescription.FragmentEntryPoint, pipelineLayout, vertexDescription);

		projectionMatrixBindGroup = CreateBindGroup(
			videoDriver,
			projectionMatrixBindGroupLayout,
			[
				new()
				{
					Binding = 0,
					Buffer = projectionMatrixBuffer.Instance,
					Size = (ulong)projectionMatrixBuffer.SizeInBytes,
				},
			]
		);

		videoDriver.WebGPU.PipelineLayoutRelease(pipelineLayout);
		videoDriver.WebGPU.BindGroupLayoutRelease(projectionMatrixBindGroupLayout);
		videoDriver.WebGPU.ShaderModuleRelease(shaderModule);
	}

	public void Dispose()
	{
		projectionMatrixBuffer.Dispose();
		videoDriver.WebGPU.BindGroupLayoutRelease(modelviewMatrixBindGroupLayout);
		videoDriver.WebGPU.RenderPipelineRelease(renderPipeline);
		videoDriver.WebGPU.BindGroupRelease(projectionMatrixBindGroup);
	}

	public void QueueWriteProjectionMatrix(Matrix4X4<float> m)
	{
		projectionMatrixBuffer.QueueWrite([m], 0);
	}

	public ModelviewMatrix CreateModelviewMatrix()
	{
		var buffer = new Buffer<Matrix4X4<float>>(videoDriver, [Matrix4X4<float>.Identity], BufferUsage.Uniform);
		return new ModelviewMatrix(
			videoDriver,
			buffer,
			CreateBindGroup(
				videoDriver,
				modelviewMatrixBindGroupLayout,
				[
					new()
					{
						Binding = 0,
						Buffer = buffer.Instance,
						Size = (ulong)buffer.SizeInBytes,
					},
				]
			)
		);
	}

	public void DrawBuffer(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix, Buffer<T> vertexBuffer, uint index, uint length)
	{
		DrawCommon(renderPassEncoder, modelviewMatrix);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, (ulong)(index * vertexBuffer.Stride), (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDraw(renderPassEncoder, length, 1, 0, 0);
	}

	public void DrawBuffers(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix, Buffer<T> vertexBuffer, Buffer<UInt16> indexBuffer, uint index, uint length)
	{
		DrawCommon(renderPassEncoder, modelviewMatrix);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, 0, (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderSetIndexBuffer(renderPassEncoder, indexBuffer.Instance, IndexFormat.Uint16, 0, (ulong)indexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDrawIndexed(renderPassEncoder, length, 1, index, 0, 0);
	}

	private void DrawCommon(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix)
	{
		videoDriver.WebGPU.RenderPassEncoderSetPipeline(renderPassEncoder, renderPipeline);
		videoDriver.WebGPU.RenderPassEncoderSetBindGroup(renderPassEncoder, 0, projectionMatrixBindGroup, 0, null);
		videoDriver.WebGPU.RenderPassEncoderSetBindGroup(renderPassEncoder, 1, modelviewMatrix.BindGroup, 0, null);
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

	private static BindGroupLayout* CreateBindGroupLayout(VideoDriver videoDriver, ReadOnlySpan<BindGroupLayoutEntry> entries)
	{
		fixed (BindGroupLayoutEntry* entryPtr = &entries[0])
		{
			var bindGroupLayoutDescriptor = new BindGroupLayoutDescriptor()
			{
				Entries = entryPtr,
				EntryCount = 1,
			};
			return videoDriver.WebGPU.DeviceCreateBindGroupLayout(videoDriver.Device, ref bindGroupLayoutDescriptor);
		}
	}

	private static PipelineLayout* CreatePipelineLayout(VideoDriver videoDriver, BindGroupLayout*[] layouts)
	{
		fixed (BindGroupLayout** layoutsPtr = &layouts[0])
		{
			var pipelineLayoutDescriptor = new PipelineLayoutDescriptor()
			{
				BindGroupLayouts = layoutsPtr,
				BindGroupLayoutCount = (nuint)layouts.Length,
			};
			return videoDriver.WebGPU.DeviceCreatePipelineLayout(videoDriver.Device, ref pipelineLayoutDescriptor);
		}
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
		return new(stride, attributes.ToArray());
	}

	private static RenderPipeline* CreateRenderPipeline(
		VideoDriver videoDriver,
		ShaderModule* shaderModule,
		string vertexEntryPoint,
		string fragmentEntryPoint,
		PipelineLayout* pipelineLayout,
		VertexDescription vertexDescription
	)
	{
		fixed (Silk.NET.WebGPU.VertexAttribute* vertexAttributePtr = &vertexDescription.Attributes[0])
		{
			var vertexEntryPointPtr = Marshal.StringToHGlobalAnsi(vertexEntryPoint);
			var vertexBufferLayout = new VertexBufferLayout()
			{
				StepMode = VertexStepMode.Vertex,
				ArrayStride = (ulong)vertexDescription.Stride,
				AttributeCount = (nuint)vertexDescription.Attributes.Length,
				Attributes = vertexAttributePtr,
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
			var fragmentEntryPointPtr = Marshal.StringToHGlobalAnsi(fragmentEntryPoint);
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
			var result = videoDriver.WebGPU.DeviceCreateRenderPipeline(videoDriver.Device, ref renderPipelineDescriptor);
			Marshal.FreeHGlobal(vertexEntryPointPtr);
			Marshal.FreeHGlobal(fragmentEntryPointPtr);
			Console.WriteLine("created render pipeline");
			return result;
		}
	}

	private static BindGroup* CreateBindGroup(VideoDriver videoDriver, BindGroupLayout* bindGroupLayout, ReadOnlySpan<BindGroupEntry> entries)
	{
		fixed (BindGroupEntry* entryPtr = &entries[0])
		{
			var bindGroupDescriptor = new BindGroupDescriptor()
			{
				Layout = bindGroupLayout,
				Entries = entryPtr,
				EntryCount = (nuint)entries.Length,
			};
			return videoDriver.WebGPU.DeviceCreateBindGroup(videoDriver.Device, ref bindGroupDescriptor);
		}
	}
}
