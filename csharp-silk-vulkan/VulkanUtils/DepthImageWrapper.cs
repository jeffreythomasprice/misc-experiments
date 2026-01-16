namespace Experiment.VulkanUtils;

using Silk.NET.Vulkan;

public sealed class DepthImageWrapper : IDisposable
{
    public readonly ImageWrapper Image;
    public readonly Format Format;

    public DepthImageWrapper(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        uint width,
        uint height
    )
    {
        Format = FindBestFormat(vk, physicalDevice);
        Image = new(
            vk,
            physicalDevice,
            device,
            commandPool,
            width,
            height,
            Format,
            ImageTiling.Optimal,
            ImageUsageFlags.DepthStencilAttachmentBit,
            MemoryPropertyFlags.DeviceLocalBit,
            ImageAspectFlags.DepthBit
        );
    }

    public void Dispose()
    {
        Image.Dispose();
    }

    public static Format FindBestFormat(Vk vk, PhysicalDeviceWrapper physicalDevice) =>
        FindSupportedFormat(
            vk,
            physicalDevice,
            [Format.D32Sfloat, Format.D32SfloatS8Uint, Format.D24UnormS8Uint],
            ImageTiling.Optimal,
            FormatFeatureFlags.DepthStencilAttachmentBit
        );

    // TODO refactor?
    private static Format FindSupportedFormat(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        IEnumerable<Format> candidates,
        ImageTiling tiling,
        FormatFeatureFlags features
    )
    {
        foreach (var format in candidates)
        {
            vk.GetPhysicalDeviceFormatProperties(
                physicalDevice.PhysicalDevice,
                format,
                out var props
            );

            if (tiling == ImageTiling.Linear && (props.LinearTilingFeatures & features) == features)
            {
                return format;
            }
            else if (
                tiling == ImageTiling.Optimal
                && (props.OptimalTilingFeatures & features) == features
            )
            {
                return format;
            }
        }

        throw new Exception("failed to find supported format");
    }
}
