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
                // TODO destroy image
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

    public void TransitionLayout(ImageLayout oldLayout, ImageLayout newLayout)
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
                    Image = Image,
                    SubresourceRange =
                    {
                        AspectMask = aspectFlags,
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

    // TODO document, assumes that buffer has bytes-per-pixel equal to this image?
    // TODO more versions, take an arbitrary source and destination rectangles
    public void CopyBufferToImage(BufferWrapper<byte> buffer)
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
                        AspectMask = aspectFlags,
                        MipLevel = 0,
                        BaseArrayLayer = 0,
                        LayerCount = 1,
                    },
                    ImageOffset = new Offset3D(0, 0, 0),
                    ImageExtent = new Extent3D((uint)Width, (uint)Height, 1),
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
    }
}
