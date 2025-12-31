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

public sealed unsafe class SwapchainWrapper : IDisposable
{
    private readonly DeviceWrapper device;
    private readonly KhrSwapchain khrSwapchain;
    private readonly SwapchainKHR swapchainKhr;
    private readonly Image[] images;
    private readonly Format format;
    private readonly Extent2D extent;

    public SwapchainWrapper(
        IWindow window,
        Vk vk,
        InstanceWrapper instance,
        SurfaceWrapper surface,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device
    )
    {
        this.device = device;

        var capabilities = surface.GetPhysicalDeviceSurfaceCapabilities(physicalDevice);
        var formats = surface.GetPhysicalDeviceSurfaceFormats(physicalDevice);
        var presentModes = surface.GetPhysicalDeviceSurfacePresentModes(physicalDevice);

        var surfaceFormat = GetBestSurfaceFormat(formats);
        var presentMode = GetBestPresentMode(presentModes);
        var extent = GetSwapExtent(window, capabilities);

        var imageCount = capabilities.MinImageCount + 1;
        if (capabilities.MaxImageCount > 0 && imageCount > capabilities.MaxImageCount)
        {
            imageCount = capabilities.MaxImageCount;
        }

        SwapchainCreateInfoKHR createInfo = new()
        {
            SType = StructureType.SwapchainCreateInfoKhr,
            Surface = surface.SurfaceKHR,
            MinImageCount = imageCount,
            ImageFormat = surfaceFormat.Format,
            ImageColorSpace = surfaceFormat.ColorSpace,
            ImageExtent = extent,
            ImageArrayLayers = 1,
            ImageUsage = ImageUsageFlags.ColorAttachmentBit,
        };

        var queueFamilyIndices = stackalloc[] {
            physicalDevice.AssertGraphicsQueueIndex(),
            physicalDevice.AssertPresentQueueIndex(),
        };

        if (queueFamilyIndices[0] != queueFamilyIndices[1])
        {
            createInfo = createInfo with
            {
                ImageSharingMode = SharingMode.Concurrent,
                QueueFamilyIndexCount = 2,
                PQueueFamilyIndices = queueFamilyIndices,
            };
        }
        else
        {
            createInfo.ImageSharingMode = SharingMode.Exclusive;
        }

        createInfo = createInfo with
        {
            PreTransform = capabilities.CurrentTransform,
            CompositeAlpha = CompositeAlphaFlagsKHR.OpaqueBitKhr,
            PresentMode = presentMode,
            Clipped = true,

            OldSwapchain = default,
        };

        if (!vk.TryGetDeviceExtension(instance.Instance, device.Device, out khrSwapchain))
        {
            throw new NotSupportedException("VK_KHR_swapchain extension not found.");
        }

        if (
            khrSwapchain.CreateSwapchain(device.Device, in createInfo, null, out swapchainKhr)
            != Result.Success
        )
        {
            throw new Exception("failed to create swap chain!");
        }

        khrSwapchain.GetSwapchainImages(device.Device, swapchainKhr, ref imageCount, null);
        images = new Image[imageCount];
        fixed (Image* swapChainImagesPtr = images)
        {
            khrSwapchain.GetSwapchainImages(
                device.Device,
                swapchainKhr,
                ref imageCount,
                swapChainImagesPtr
            );
        }

        format = surfaceFormat.Format;
        this.extent = extent;
    }

    public void Dispose()
    {
        khrSwapchain.DestroySwapchain(device.Device, swapchainKhr, null);
    }

    private static SurfaceFormatKHR GetBestSurfaceFormat(IReadOnlyList<SurfaceFormatKHR> formats)
    {
        return formats
            .OrderBy(x =>
            {
                if (
                    x.Format == Format.B8G8R8A8Srgb
                    && x.ColorSpace == ColorSpaceKHR.SpaceSrgbNonlinearKhr
                )
                {
                    return 0;
                }
                return 1;
            })
            .First();
    }

    private static PresentModeKHR GetBestPresentMode(IReadOnlyList<PresentModeKHR> presentModes)
    {
        return presentModes
            .OrderBy(x =>
                x switch
                {
                    PresentModeKHR.ImmediateKhr => 0,
                    PresentModeKHR.FifoRelaxedKhr => 1,
                    PresentModeKHR.FifoKhr => 2,
                    PresentModeKHR.MailboxKhr => 3,
                    _ => 4,
                }
            )
            .First();
    }

    private static Extent2D GetSwapExtent(IWindow window, SurfaceCapabilitiesKHR capabilities)
    {
        if (capabilities.CurrentExtent.Width != uint.MaxValue)
        {
            return capabilities.CurrentExtent;
        }
        else
        {
            var framebufferSize = window.FramebufferSize;

            Extent2D actualExtent = new()
            {
                Width = (uint)framebufferSize.X,
                Height = (uint)framebufferSize.Y,
            };

            actualExtent.Width = Math.Clamp(
                actualExtent.Width,
                capabilities.MinImageExtent.Width,
                capabilities.MaxImageExtent.Width
            );
            actualExtent.Height = Math.Clamp(
                actualExtent.Height,
                capabilities.MinImageExtent.Height,
                capabilities.MaxImageExtent.Height
            );

            return actualExtent;
        }
    }
}
