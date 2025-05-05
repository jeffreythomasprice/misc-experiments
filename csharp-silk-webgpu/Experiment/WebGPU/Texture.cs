namespace Experiment.WebGPU;

using System.Dynamic;
using System.Reflection;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

public unsafe class Texture : IDisposable
{
	public class Description : IDisposable
	{
		public required VideoDriver VideoDriver { get; init; }
		public required BindGroupLayout* BindGroupLayout { get; init; }
		public required bool ReleaseBindGroupLayout { get; init; }
		public required uint TextureBinding { get; init; }
		public required uint SamplerBinding { get; init; }

		public void Dispose()
		{
			if (ReleaseBindGroupLayout)
			{
				VideoDriver.WebGPU.BindGroupLayoutRelease(BindGroupLayout);
			}
		}
	}

	public interface IDescriptionSource
	{
		public Description TextureDescription { get; }
	}

	private readonly VideoDriver videoDriver;
	private readonly Vector2D<int> size;
	private readonly Silk.NET.WebGPU.Texture* texture;
	private readonly BindGroup* bindGroup;

	public Texture(Description description, Vector2D<int> size)
	{
		try
		{
			this.videoDriver = description.VideoDriver;
			this.size = size;

			var textureDescriptor = new TextureDescriptor()
			{
				Size = new()
				{
					Width = (uint)size.X,
					Height = (uint)size.Y,
					DepthOrArrayLayers = 1,
				},
				MipLevelCount = 1,
				SampleCount = 1,
				Dimension = TextureDimension.Dimension2D,
				Format = TextureFormat.Rgba8Unorm,
				Usage = TextureUsage.TextureBinding | TextureUsage.CopyDst,
			};
			texture = videoDriver.WebGPU.DeviceCreateTexture(videoDriver.Device, ref textureDescriptor);

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
				description.BindGroupLayout,
				[
					new()
					{
						Binding = description.TextureBinding,
						TextureView = textureView,
					},
					new()
					{
						Binding = description.SamplerBinding,
						Sampler = sampler,
					},
				]
			);

			videoDriver.WebGPU.TextureViewRelease(textureView);
			videoDriver.WebGPU.SamplerRelease(sampler);
		}
		finally
		{
			description.Dispose();
		}
	}

	internal BindGroup* BindGroup => bindGroup;

	public void Dispose()
	{
		videoDriver.WebGPU.TextureRelease(texture);
		videoDriver.WebGPU.BindGroupRelease(bindGroup);
	}

	public Vector2D<int> Size => size;

	public void QueueUpdate(SixLabors.ImageSharp.Image image, Vector2D<int> origin)
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

public static class TextureExtensions
{
	public static Texture CreateTexture(this Texture.IDescriptionSource source, Vector2D<int> size)
	{
		return new Texture(source.TextureDescription, size);
	}

	public static Texture CreateTexture(this Texture.IDescriptionSource source, SixLabors.ImageSharp.Image image)
	{
		var texture = source.CreateTexture(new Vector2D<int>(image.Width, image.Height));
		texture.QueueUpdate(image, new(0, 0));
		return texture;
	}

	public static Texture CreateTexture(this Texture.IDescriptionSource source, Stream stream)
	{
		using var image = SixLabors.ImageSharp.Image.Load(stream)
			?? throw new NullReferenceException("failed to load image from stream");
		return source.CreateTexture(image);
	}

	public static Texture CreateTextureFromManifestResource(this Texture.IDescriptionSource source, Assembly assembly, string name)
	{
		using var stream = assembly.AssertManifestResourceStream(name);
		return source.CreateTexture(stream);
	}

	public static Texture CreateTextureFromManifestResource(this Texture.IDescriptionSource source, string name)
	{
		return source.CreateTextureFromManifestResource(Assembly.GetExecutingAssembly(), name);
	}
}
