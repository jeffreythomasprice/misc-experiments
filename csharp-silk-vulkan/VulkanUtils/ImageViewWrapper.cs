namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class ImageViewWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    public readonly ImageView ImageView;

    public ImageViewWrapper(Vk vk, DeviceWrapper device, Format format, Image image)
    {
        this.vk = vk;
        this.device = device;

        ImageViewCreateInfo createInfo = new()
        {
            SType = StructureType.ImageViewCreateInfo,
            Image = image,
            ViewType = ImageViewType.Type2D,
            Format = format,
            Components =
            {
                R = ComponentSwizzle.Identity,
                G = ComponentSwizzle.Identity,
                B = ComponentSwizzle.Identity,
                A = ComponentSwizzle.Identity,
            },
            SubresourceRange =
            {
                AspectMask = ImageAspectFlags.ColorBit,
                BaseMipLevel = 0,
                LevelCount = 1,
                BaseArrayLayer = 0,
                LayerCount = 1,
            },
        };

        if (vk.CreateImageView(device.Device, in createInfo, null, out ImageView) != Result.Success)
        {
            throw new Exception("failed to create image views");
        }
    }

    public void Dispose()
    {
        vk.DestroyImageView(device.Device, ImageView, null);
    }
}
