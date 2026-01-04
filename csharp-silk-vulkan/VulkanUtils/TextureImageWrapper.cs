namespace Experiment.VulkanUtils;

using Microsoft.Extensions.Logging;
using Silk.NET.Vulkan;

public sealed unsafe class TextureImageWrapper : IDisposable
{
    private readonly ILogger log;

    private readonly Vk vk;
    private readonly DeviceWrapper device;

    private readonly Image image;
    private readonly DeviceMemory deviceMemory;

    public TextureImageWrapper(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        SixLabors.ImageSharp.Image<SixLabors.ImageSharp.PixelFormats.Rgba32> source
    )
    {
        log = LoggerUtils.Factory.Value.CreateLogger(GetType());

        this.vk = vk;
        this.device = device;

        using var buffer = new BufferWrapper<byte>(
            vk,
            physicalDevice,
            device,
            source.CopyPixelDataTo,
            (UInt64)(source.Width * source.Height * source.PixelType.BitsPerPixel / 8),
            BufferUsageFlags.TransferSrcBit
        );

        CreateImage(
            vk,
            physicalDevice,
            device,
            (uint)source.Width,
            (uint)source.Height,
            Format.R8G8B8A8Srgb,
            ImageTiling.Optimal,
            ImageUsageFlags.TransferDstBit | ImageUsageFlags.SampledBit,
            MemoryPropertyFlags.DeviceLocalBit,
            ref image,
            ref deviceMemory
        );
        TransitionImageLayout(
            vk,
            device,
            commandPool,
            image,
            ImageLayout.Undefined,
            ImageLayout.TransferDstOptimal
        );
        CopyBufferToImage(
            vk,
            device,
            commandPool,
            buffer,
            image,
            (uint)source.Width,
            (uint)source.Height
        );
        TransitionImageLayout(
            vk,
            device,
            commandPool,
            image,
            ImageLayout.TransferDstOptimal,
            ImageLayout.ShaderReadOnlyOptimal
        );
    }

    public void Dispose()
    {
        vk.DestroyImage(device.Device, image, null);
        vk.FreeMemory(device.Device, deviceMemory, null);
    }

    // TODO the rest of the texture stuff

    private void CreateImage(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        uint width,
        uint height,
        Format format,
        ImageTiling tiling,
        ImageUsageFlags usage,
        MemoryPropertyFlags properties,
        ref Image image,
        ref DeviceMemory imageMemory
    )
    {
        var imageInfo = new ImageCreateInfo()
        {
            SType = StructureType.ImageCreateInfo,
            ImageType = ImageType.Type2D,
            Extent =
            {
                Width = width,
                Height = height,
                Depth = 1,
            },
            MipLevels = 1,
            ArrayLayers = 1,
            Format = format,
            Tiling = tiling,
            InitialLayout = ImageLayout.Undefined,
            Usage = usage,
            Samples = SampleCountFlags.Count1Bit,
            SharingMode = SharingMode.Exclusive,
        };

        fixed (Image* imagePtr = &image)
        {
            if (vk.CreateImage(device.Device, in imageInfo, null, imagePtr) != Result.Success)
            {
                throw new Exception("failed to create image");
            }
        }

        vk.GetImageMemoryRequirements(device.Device, image, out var memRequirements);

        var allocInfo = new MemoryAllocateInfo()
        {
            SType = StructureType.MemoryAllocateInfo,
            AllocationSize = memRequirements.Size,
            MemoryTypeIndex = physicalDevice.FindMemoryType(
                memRequirements.MemoryTypeBits,
                properties
            ),
        };

        fixed (DeviceMemory* imageMemoryPtr = &imageMemory)
        {
            if (
                vk.AllocateMemory(device.Device, in allocInfo, null, imageMemoryPtr)
                != Result.Success
            )
            {
                throw new Exception("failed to allocate image memory");
            }
        }

        vk.BindImageMemory(device.Device, image, imageMemory, 0);
    }

    private void TransitionImageLayout(
        Vk vk,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        Image image,
        ImageLayout oldLayout,
        ImageLayout newLayout
    )
    {
        CommandBufferWrapper.OneTimeSubmit(
            vk,
            device,
            commandPool,
            (commandBuffer) =>
            {
                var barrier = new ImageMemoryBarrier()
                {
                    SType = StructureType.ImageMemoryBarrier,
                    OldLayout = oldLayout,
                    NewLayout = newLayout,
                    SrcQueueFamilyIndex = Vk.QueueFamilyIgnored,
                    DstQueueFamilyIndex = Vk.QueueFamilyIgnored,
                    Image = image,
                    SubresourceRange =
                    {
                        AspectMask = ImageAspectFlags.ColorBit,
                        BaseMipLevel = 0,
                        LevelCount = 1,
                        BaseArrayLayer = 0,
                        LayerCount = 1,
                    },
                };

                PipelineStageFlags sourceStage;
                PipelineStageFlags destinationStage;

                if (
                    oldLayout == ImageLayout.Undefined
                    && newLayout == ImageLayout.TransferDstOptimal
                )
                {
                    barrier.SrcAccessMask = 0;
                    barrier.DstAccessMask = AccessFlags.TransferWriteBit;

                    sourceStage = PipelineStageFlags.TopOfPipeBit;
                    destinationStage = PipelineStageFlags.TransferBit;
                }
                else if (
                    oldLayout == ImageLayout.TransferDstOptimal
                    && newLayout == ImageLayout.ShaderReadOnlyOptimal
                )
                {
                    barrier.SrcAccessMask = AccessFlags.TransferWriteBit;
                    barrier.DstAccessMask = AccessFlags.ShaderReadBit;

                    sourceStage = PipelineStageFlags.TransferBit;
                    destinationStage = PipelineStageFlags.FragmentShaderBit;
                }
                else
                {
                    throw new Exception("unsupported layout transition");
                }

                vk.CmdPipelineBarrier(
                    commandBuffer.CommandBuffer,
                    sourceStage,
                    destinationStage,
                    0,
                    0,
                    null,
                    0,
                    null,
                    1,
                    in barrier
                );
            }
        );
    }

    private static void CopyBufferToImage(
        Vk vk,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        BufferWrapper<byte> buffer,
        Image image,
        uint width,
        uint height
    )
    {
        CommandBufferWrapper.OneTimeSubmit(
            vk,
            device,
            commandPool,
            (commandBuffer) =>
            {
                var region = new BufferImageCopy()
                {
                    BufferOffset = 0,
                    BufferRowLength = 0,
                    BufferImageHeight = 0,
                    ImageSubresource =
                    {
                        AspectMask = ImageAspectFlags.ColorBit,
                        MipLevel = 0,
                        BaseArrayLayer = 0,
                        LayerCount = 1,
                    },
                    ImageOffset = new Offset3D(0, 0, 0),
                    ImageExtent = new Extent3D(width, height, 1),
                };

                vk.CmdCopyBufferToImage(
                    commandBuffer.CommandBuffer,
                    buffer.Buffer,
                    image,
                    ImageLayout.TransferDstOptimal,
                    1,
                    in region
                );
            }
        );
    }
}
