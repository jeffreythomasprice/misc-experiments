namespace Experiment.VulkanUtils;

using Silk.NET.Vulkan;

public sealed unsafe class ImageWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly CommandPoolWrapper commandPool;

    private readonly SixLabors.ImageSharp.Size size;
    private readonly ImageAspectFlags aspectFlags;

    public readonly Image Image;
    private readonly DeviceMemory memory;

    public ImageWrapper(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        uint width,
        uint height,
        Format format,
        ImageTiling tiling,
        ImageUsageFlags usage,
        MemoryPropertyFlags properties,
        ImageAspectFlags aspectFlags
    )
    {
        this.vk = vk;
        this.device = device;
        this.commandPool = commandPool;

        size = new((int)width, (int)height);
        this.aspectFlags = aspectFlags;

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

        fixed (Image* imagePtr = &Image)
        {
            if (vk.CreateImage(device.Device, in imageInfo, null, imagePtr) != Result.Success)
            {
                throw new Exception("failed to create image");
            }
        }

        vk.GetImageMemoryRequirements(device.Device, Image, out var memRequirements);

        var allocInfo = new MemoryAllocateInfo()
        {
            SType = StructureType.MemoryAllocateInfo,
            AllocationSize = memRequirements.Size,
            MemoryTypeIndex = physicalDevice.FindMemoryType(
                memRequirements.MemoryTypeBits,
                properties
            ),
        };

        fixed (DeviceMemory* imageMemoryPtr = &memory)
        {
            if (
                vk.AllocateMemory(device.Device, in allocInfo, null, imageMemoryPtr)
                != Result.Success
            )
            {
                vk.DestroyImage(device.Device, Image, null);
                throw new Exception("failed to allocate image memory");
            }
        }

        vk.BindImageMemory(device.Device, image: Image, memory, 0);
    }

    public void Dispose()
    {
        vk.DestroyImage(device.Device, Image, null);
        vk.FreeMemory(device.Device, memory, null);
    }

    public SixLabors.ImageSharp.Size Size => size;
    public int Width => size.Width;
    public int Height => size.Height;

    public ImageAspectFlags AspectFlags => aspectFlags;

    /// <param name="buffer"></param>
    /// <param name="bufferOffset">byte index where the first texel in the image is</param>
    /// <param name="bufferRowLength">how many texels to skip between rows; i.e. the stride of a row in texels, not bytes</param>
    /// <param name="bufferImageHeight">how many texels tall is the image in the buffer</param>
    /// <param name="imageOffset">the corner of the image stored in the buffer to start copying from</param>
    /// <param name="imageExtent">the width and height of the portion of the image to copy</param>
    public void CopyBufferToImage(
        BufferWrapper<byte> buffer,
        UInt64 bufferOffset,
        UInt32 bufferRowLength,
        UInt32 bufferImageHeight,
        Offset2D imageOffset,
        Extent2D imageExtent
    )
    {
        CommandBufferWrapper.OneTimeSubmit(
            vk,
            device,
            commandPool,
            (commandBuffer) =>
            {
                TransitionLayout(
                    commandBuffer,
                    ImageLayout.Undefined,
                    ImageLayout.TransferDstOptimal,
                    AccessFlags.None,
                    AccessFlags.TransferWriteBit,
                    PipelineStageFlags.TopOfPipeBit,
                    PipelineStageFlags.TransferBit
                );
            }
        );

        CommandBufferWrapper.OneTimeSubmit(
            vk,
            device,
            commandPool,
            (commandBuffer) =>
            {
                var region = new BufferImageCopy()
                {
                    BufferOffset = bufferOffset,
                    BufferRowLength = bufferRowLength,
                    BufferImageHeight = bufferImageHeight,
                    ImageSubresource =
                    {
                        AspectMask = aspectFlags,
                        MipLevel = 0,
                        BaseArrayLayer = 0,
                        LayerCount = 1,
                    },
                    ImageOffset = new Offset3D(imageOffset.X, imageOffset.Y, 0),
                    ImageExtent = new Extent3D(
                        (uint)imageExtent.Width,
                        (uint)imageExtent.Height,
                        1
                    ),
                };

                vk.CmdCopyBufferToImage(
                    commandBuffer.CommandBuffer,
                    buffer.Buffer,
                    Image,
                    ImageLayout.TransferDstOptimal,
                    1,
                    in region
                );
            }
        );

        CommandBufferWrapper.OneTimeSubmit(
            vk,
            device,
            commandPool,
            (commandBuffer) =>
            {
                TransitionLayout(
                    commandBuffer,
                    ImageLayout.TransferDstOptimal,
                    ImageLayout.ShaderReadOnlyOptimal,
                    AccessFlags.TransferWriteBit,
                    AccessFlags.ShaderReadBit,
                    PipelineStageFlags.TransferBit,
                    PipelineStageFlags.FragmentShaderBit
                );
            }
        );
    }

    private void TransitionLayout(
        CommandBufferWrapper commandBuffer,
        ImageLayout oldLayout,
        ImageLayout newLayout,
        AccessFlags srcAccessMask,
        AccessFlags dstAccessMask,
        PipelineStageFlags sourceStage,
        PipelineStageFlags destinationStage
    )
    {
        var barrier = new ImageMemoryBarrier()
        {
            SType = StructureType.ImageMemoryBarrier,
            OldLayout = oldLayout,
            NewLayout = newLayout,
            SrcQueueFamilyIndex = Vk.QueueFamilyIgnored,
            DstQueueFamilyIndex = Vk.QueueFamilyIgnored,
            Image = Image,
            SubresourceRange =
            {
                AspectMask = aspectFlags,
                BaseMipLevel = 0,
                LevelCount = 1,
                BaseArrayLayer = 0,
                LayerCount = 1,
            },
            SrcAccessMask = srcAccessMask,
            DstAccessMask = dstAccessMask,
        };

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
}
