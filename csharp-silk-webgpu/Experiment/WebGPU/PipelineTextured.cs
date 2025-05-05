namespace Experiment.WebGPU;

using System.Numerics;
using System.Runtime.InteropServices;
using Silk.NET.Maths;
using Silk.NET.WebGPU;
using SixLabors.ImageSharp.Advanced;
using SixLabors.ImageSharp.Processing;

// TODO make inner struct of pipeline
public struct VertexTextured
{
	[Experiment.WebGPU.VertexAttribute(Format = VertexFormat.Float32x2, ShaderLocation = 0)]
	public readonly Vector2D<float> Position;

	[Experiment.WebGPU.VertexAttribute(Format = VertexFormat.Float32x4, ShaderLocation = 1)]
	public readonly Vector2D<float> TextureCoordinate;

	[Experiment.WebGPU.VertexAttribute(Format = VertexFormat.Float32x4, ShaderLocation = 2)]
	public readonly Vector4D<float> Color;

	public VertexTextured(Vector2D<float> position, Vector2D<float> textureCoordinate, Vector4D<float> color)
	{
		this.Position = position;
		this.TextureCoordinate = textureCoordinate;
		this.Color = color;
	}
}

public unsafe class PipelineTextured : Pipeline<VertexTextured>
{
	public class Texture : IDisposable
	{
		private readonly VideoDriver videoDriver;
		private readonly Vector2D<int> size;
		private readonly Silk.NET.WebGPU.Texture* texture;
		private readonly BindGroup* bindGroup;

		internal Texture(VideoDriver videoDriver, BindGroupLayout* bindGroupLayout, Vector2D<int> size)
		{
			this.videoDriver = videoDriver;
			this.size = size;

			texture = videoDriver.CreateTexture(size);

			var textureViewDescriptor = new TextureViewDescriptor()
			{
				Dimension = TextureViewDimension.Dimension2D,
				BaseMipLevel = 0,
				MipLevelCount = 1,
				BaseArrayLayer = 0,
				ArrayLayerCount = 1,
				Aspect = TextureAspect.All,
			};
			var textureView = videoDriver.WebGPU.TextureCreateView(texture, ref textureViewDescriptor);

			var samplerDescriptor = new SamplerDescriptor()
			{
				AddressModeU = AddressMode.ClampToEdge,
				AddressModeV = AddressMode.ClampToEdge,
				AddressModeW = AddressMode.ClampToEdge,
				MagFilter = FilterMode.Linear,
				MinFilter = FilterMode.Linear,
				MipmapFilter = MipmapFilterMode.Nearest,
				LodMinClamp = 1,
				LodMaxClamp = 1,
				Compare = CompareFunction.Undefined,
				MaxAnisotropy = 1,
			};
			var sampler = videoDriver.WebGPU.DeviceCreateSampler(videoDriver.Device, ref samplerDescriptor);

			bindGroup = videoDriver.CreateBindGroup(
				bindGroupLayout,
				[
					new()
					{
						// TODO use const
						Binding = 0,
						TextureView = textureView,
					},
					new()
					{
						// TODO use const
						Binding = 1,
						Sampler = sampler,
					},
				]
			);

			videoDriver.WebGPU.TextureViewRelease(textureView);
			videoDriver.WebGPU.SamplerRelease(sampler);
		}

		internal BindGroup* BindGroup => bindGroup;

		public void Dispose()
		{
			videoDriver.WebGPU.TextureRelease(texture);
			videoDriver.WebGPU.BindGroupRelease(bindGroup);
		}

		public Vector2D<int> Size => size;

		internal void QueueUpdate(SixLabors.ImageSharp.Image image, Vector2D<int> origin)
		{
			var imageConfiguration = image.Configuration.Clone();
			imageConfiguration.PreferContiguousImageBuffers = true;
			var imageInTheRightFormat = image.CloneAs<SixLabors.ImageSharp.PixelFormats.Rgba32>(imageConfiguration);
			if (!imageInTheRightFormat.DangerousTryGetSinglePixelMemory(out var memory))
			{
				throw new Exception("failed to get contiguous memory block for image");
			}
			using var memoryHandle = memory.Pin();
			var imageCopyTexture = new ImageCopyTexture()
			{
				Texture = texture,
				MipLevel = 0,
				Origin = new()
				{
					X = (uint)origin.X,
					Y = (uint)origin.Y,
					Z = 0,
				},
				Aspect = TextureAspect.All,
			};
			var rowLengthInBytes = imageInTheRightFormat.Width * 4;
			var dataLayout = new TextureDataLayout()
			{
				Offset = 0,
				BytesPerRow = (uint)rowLengthInBytes,
				RowsPerImage = (uint)imageInTheRightFormat.Height,
			};
			var writeSize = new Extent3D()
			{
				Width = (uint)imageInTheRightFormat.Width,
				Height = (uint)imageInTheRightFormat.Height,
				DepthOrArrayLayers = 1,
			};
			videoDriver.WebGPU.QueueWriteTexture(
				videoDriver.Queue,
				ref imageCopyTexture,
				memoryHandle.Pointer,
				(nuint)(imageInTheRightFormat.Height * rowLengthInBytes),
				ref dataLayout,
				ref writeSize
			);
		}
	}

	public PipelineTextured(VideoDriver videoDriver) : base(
			videoDriver,
			new()
			{
				Source = App.EmbeddedFileAsString("Experiment.Assets.Shaders.shaderTextured.wgsl"),
				VertexEntryPoint = "vs_main",
				FragmentEntryPoint = "fs_main",
			},
			[
				CreateTexturedBindGroupLayout(videoDriver),
			]
		)
	{ }

	public Texture CreateTexture(Vector2D<int> size)
	{
		var layout = CreateTexturedBindGroupLayout(videoDriver);
		var result = new Texture(videoDriver, layout, size);
		videoDriver.WebGPU.BindGroupLayoutRelease(layout);
		return result;
	}

	public Texture CreateTexture(SixLabors.ImageSharp.Image image)
	{
		var layout = CreateTexturedBindGroupLayout(videoDriver);
		var result = new Texture(videoDriver, layout, new(image.Width, image.Height));
		result.QueueUpdate(image, new(0, 0));
		videoDriver.WebGPU.BindGroupLayoutRelease(layout);
		return result;
	}

	public Texture CreateTexture(Stream stream)
	{
		using var image = SixLabors.ImageSharp.Image.Load(stream)
			?? throw new NullReferenceException("failed to load image from stream");
		return CreateTexture(image);
	}

	public Texture CreateTextureFromEmbeddedFile(string name)
	{
		using var stream = App.EmbeddedFileAsStream(name);
		return CreateTexture(stream);
	}

	public void DrawBuffer(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix, Texture texture, Buffer<VertexTextured> vertexBuffer, uint index, uint length)
	{
		DrawCommon(renderPassEncoder, modelviewMatrix, texture);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, (ulong)(index * vertexBuffer.Stride), (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDraw(renderPassEncoder, length, 1, 0, 0);
	}

	public void DrawBuffers(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix, Texture texture, Buffer<VertexTextured> vertexBuffer, Buffer<UInt16> indexBuffer, uint index, uint length)
	{
		DrawCommon(renderPassEncoder, modelviewMatrix, texture);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, 0, (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderSetIndexBuffer(renderPassEncoder, indexBuffer.Instance, IndexFormat.Uint16, 0, (ulong)indexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDrawIndexed(renderPassEncoder, length, 1, index, 0, 0);
	}

	private void DrawCommon(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix, Texture texture)
	{
		base.DrawCommon(renderPassEncoder, modelviewMatrix);
		videoDriver.WebGPU.RenderPassEncoderSetBindGroup(
			renderPassEncoder,
			// TODO use const
			2,
			texture.BindGroup,
			0,
			null
		);
	}

	private static BindGroupLayout* CreateTexturedBindGroupLayout(VideoDriver videoDriver)
	{
		return videoDriver.CreateBindGroupLayout([
			new BindGroupLayoutEntry()
			{
				// TODO use const
				Binding = 0,
				Visibility = ShaderStage.Fragment,
				Texture = new()
				{
					Multisampled = false,
					ViewDimension = TextureViewDimension.Dimension2D,
					SampleType = TextureSampleType.Float,
				}
			},
			new BindGroupLayoutEntry()
			{
				// TODO use const
				Binding = 1,
				Visibility = ShaderStage.Fragment,
				Sampler = new()
				{
					Type = SamplerBindingType.Filtering,
				},
			},
		]);
	}
}