using System.Runtime.InteropServices;
using Experiment.WebGPU;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

public unsafe static class VideoDriverExtensions
{
	public static ShaderModule* CreateShader(this VideoDriver videoDriver, ShaderDescription shaderDescription)
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

	public static BindGroupLayout* CreateBindGroupLayout(this VideoDriver videoDriver, ReadOnlySpan<BindGroupLayoutEntry> entries)
	{
		fixed (BindGroupLayoutEntry* entryPtr = &entries[0])
		{
			var bindGroupLayoutDescriptor = new BindGroupLayoutDescriptor()
			{
				Entries = entryPtr,
				EntryCount = (nuint)entries.Length,
			};
			return videoDriver.WebGPU.DeviceCreateBindGroupLayout(videoDriver.Device, ref bindGroupLayoutDescriptor);
		}
	}

	public static PipelineLayout* CreatePipelineLayout(this VideoDriver videoDriver, BindGroupLayout*[] layouts)
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

	public static RenderPipeline* CreateRenderPipeline(
		this VideoDriver videoDriver,
		string? label,
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
			IntPtr? labelPtr = null;
			if (label != null)
			{
				labelPtr = Marshal.StringToHGlobalAnsi(label);
				renderPipelineDescriptor.Label = (byte*)labelPtr;
			}
			var result = videoDriver.WebGPU.DeviceCreateRenderPipeline(videoDriver.Device, ref renderPipelineDescriptor);
			Marshal.FreeHGlobal(vertexEntryPointPtr);
			Marshal.FreeHGlobal(fragmentEntryPointPtr);
			if (labelPtr != null)
			{
				Marshal.FreeHGlobal(labelPtr.Value);
			}
			if (label == null)
			{
				Console.WriteLine("created render pipeline");
			}
			else
			{
				Console.WriteLine($"created render pipeline: {label}");
			}
			return result;
		}
	}

	public static BindGroup* CreateBindGroup(this VideoDriver videoDriver, BindGroupLayout* bindGroupLayout, ReadOnlySpan<BindGroupEntry> entries)
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