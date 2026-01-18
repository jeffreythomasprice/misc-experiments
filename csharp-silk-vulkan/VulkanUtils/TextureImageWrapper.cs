namespace Experiment.VulkanUtils;

using Microsoft.Extensions.Logging;
using Silk.NET.Maths;
using Silk.NET.Vulkan;

public sealed unsafe class TextureImageWrapper : IDisposable
{
    private static readonly Lazy<ILogger> log = new(() =>
        LoggerUtils.Factory.Value.CreateLogger<TextureImageWrapper>()
    );

    private readonly Vk vk;
    private readonly PhysicalDeviceWrapper physicalDevice;
    private readonly DeviceWrapper device;

    private readonly ImageWrapper image;

    public readonly ImageViewWrapper ImageView;
    public readonly Sampler Sampler;

    public static TextureImageWrapper LoadFromImageAtPath(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        string path
    )
    {
        using var source =
            SixLabors.ImageSharp.Image.Load<SixLabors.ImageSharp.PixelFormats.Rgba32>(path);
        log.Value.LogTrace(
            "loaded texture image from path: {Path}, size: {Width}x{Height}, bits per pixel: {BitsPerPixel}, alpha: {Alpha}",
            path,
            source.Width,
            source.Height,
            source.PixelType.BitsPerPixel,
            source.PixelType.AlphaRepresentation
        );
        return new TextureImageWrapper(vk, physicalDevice, device, commandPool, source);
    }

    public TextureImageWrapper(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        SixLabors.ImageSharp.Image<SixLabors.ImageSharp.PixelFormats.Rgba32> source
    )
    {
        this.vk = vk;
        this.physicalDevice = physicalDevice;
        this.device = device;

        using var buffer = new BufferWrapper<byte>(
            vk,
            physicalDevice,
            device,
            source.Width * source.Height * source.PixelType.BytesPerPixel,
            BufferUsageFlags.TransferSrcBit,
            source.CopyPixelDataTo
        );

        image = new(
            vk,
            physicalDevice,
            device,
            commandPool,
            (uint)source.Width,
            (uint)source.Height,
            Format.R8G8B8A8Srgb,
            ImageTiling.Optimal,
            ImageUsageFlags.TransferDstBit | ImageUsageFlags.SampledBit,
            MemoryPropertyFlags.DeviceLocalBit,
            ImageAspectFlags.ColorBit
        );
        // TODO just call CopyImageToTexture instead?
        image.CopyBufferToImage(
            buffer,
            0,
            (UInt32)source.Width,
            (UInt32)source.Height,
            new(0, 0),
            new((UInt32)source.Width, (UInt32)source.Height)
        );

        ImageView = new ImageViewWrapper(vk, device, Format.R8G8B8A8Srgb, image.Image);

        Sampler = CreateTextureSampler(vk, physicalDevice, device);
    }

    public void Dispose()
    {
        vk.DestroySampler(device.Device, Sampler, null);
        ImageView.Dispose();
        image.Dispose();
    }

    public SixLabors.ImageSharp.Size Size => image.Size;
    public int Width => Size.Width;
    public int Height => Size.Height;

    public void CopyImageToTexture(
        SixLabors.ImageSharp.Image<SixLabors.ImageSharp.PixelFormats.Rgba32> source,
        Rectangle<int> sourceBounds,
        Vector2D<int> destination
    )
    {
        var sourceSizeRect = new Rectangle<int>(0, 0, source.Width, source.Height);
        if (!sourceSizeRect.Contains(sourceBounds))
        {
            throw new ArgumentOutOfRangeException(
                nameof(sourceBounds),
                "sourceBounds must be within the dimensions of the source image"
            );
        }

        var destinationSizeRect = new Rectangle<int>(0, 0, Width, Height);
        var destinationBounds = new Rectangle<int>(
            destination.X,
            destination.Y,
            sourceBounds.Size.X,
            sourceBounds.Size.Y
        );
        if (!destinationSizeRect.Contains(destinationBounds))
        {
            throw new ArgumentOutOfRangeException(
                nameof(destination),
                "destination and sourceBounds size must form a rectangle that is within the dimensions of the texture image"
            );
        }

        using var buffer = new BufferWrapper<byte>(
            vk,
            physicalDevice,
            device,
            source.Width * source.Height * source.PixelType.BytesPerPixel,
            BufferUsageFlags.TransferSrcBit,
            source.CopyPixelDataTo
        );

        image.CopyBufferToImage(
            buffer,
            (UInt64)source.Width
                * (UInt64)source.PixelType.BytesPerPixel
                * (UInt64)sourceBounds.Origin.Y
                + (UInt64)source.PixelType.BytesPerPixel * (UInt64)sourceBounds.Origin.X,
            (UInt32)source.Width,
            (UInt32)source.Height,
            new(destination.X, destination.Y),
            new((UInt32)sourceBounds.Size.X, (UInt32)sourceBounds.Size.Y)
        );
    }

    private static Sampler CreateTextureSampler(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device
    )
    {
        vk.GetPhysicalDeviceProperties(physicalDevice.PhysicalDevice, out var properties);

        var samplerInfo = new SamplerCreateInfo()
        {
            SType = StructureType.SamplerCreateInfo,
            MagFilter = Filter.Linear,
            MinFilter = Filter.Linear,
            AddressModeU = SamplerAddressMode.Repeat,
            AddressModeV = SamplerAddressMode.Repeat,
            AddressModeW = SamplerAddressMode.Repeat,
            AnisotropyEnable = true,
            MaxAnisotropy = properties.Limits.MaxSamplerAnisotropy,
            BorderColor = BorderColor.IntOpaqueBlack,
            UnnormalizedCoordinates = false,
            CompareEnable = false,
            CompareOp = CompareOp.Always,
            MipmapMode = SamplerMipmapMode.Linear,
        };

        if (vk.CreateSampler(device.Device, in samplerInfo, null, out var result) != Result.Success)
        {
            throw new Exception("failed to create texture sampler");
        }
        return result;
    }
}
