namespace Experiment.VulkanUtils;

using Microsoft.Extensions.Logging;
using Silk.NET.Vulkan;

public sealed unsafe class TextureImageWrapper : IDisposable
{
    private static readonly Lazy<ILogger> log = new(() =>
        LoggerUtils.Factory.Value.CreateLogger<TextureImageWrapper>()
    );

    private readonly Vk vk;
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
            SixLabors.ImageSharp.Image.Load<SixLabors.ImageSharp.PixelFormats.Rgba32>(
                "Resources/silk.png"
            );
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
        this.device = device;

        using var buffer = new BufferWrapper<byte>(
            vk,
            physicalDevice,
            device,
            source.Width * source.Height * source.PixelType.BitsPerPixel / 8,
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
        image.TransitionLayout(ImageLayout.Undefined, ImageLayout.TransferDstOptimal);
        image.CopyBufferToImage(buffer);
        image.TransitionLayout(ImageLayout.TransferDstOptimal, ImageLayout.ShaderReadOnlyOptimal);

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
