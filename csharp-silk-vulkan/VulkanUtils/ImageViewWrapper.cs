namespace Experiment.VulkanUtils;

using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Experiment.VulkanUtils;
using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Core.Native;
using Silk.NET.Maths;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.EXT;
using Silk.NET.Vulkan.Extensions.KHR;
using Silk.NET.Windowing;

public sealed unsafe class ImageViewWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly ImageView imageView;

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

        if (
            vk!.CreateImageView(device.Device, in createInfo, null, out imageView) != Result.Success
        )
        {
            throw new Exception("failed to create image views!");
        }
    }

    public void Dispose()
    {
        vk.DestroyImageView(device.Device, imageView, null);
    }
}
