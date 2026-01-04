namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.KHR;
using Silk.NET.Windowing;

public sealed unsafe class SwapchainWrapper : IDisposable
{
    private readonly DeviceWrapper device;
    public readonly KhrSwapchain KhrSwapchain;
    public readonly SwapchainKHR SwapchainKhr;
    public readonly Image[] Images;
    public readonly Format Format;
    public readonly Extent2D Extent;

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

        var createInfo = new SwapchainCreateInfoKHR()
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
        };

        if (!vk.TryGetDeviceExtension(instance.Instance, device.Device, out KhrSwapchain))
        {
            throw new NotSupportedException("VK_KHR_swapchain extension not found.");
        }

        if (
            KhrSwapchain.CreateSwapchain(device.Device, in createInfo, null, out SwapchainKhr)
            != Result.Success
        )
        {
            throw new Exception("failed to create swap chain");
        }

        KhrSwapchain.GetSwapchainImages(device.Device, SwapchainKhr, ref imageCount, null);
        Images = new Image[imageCount];
        fixed (Image* swapChainImagesPtr = Images)
        {
            KhrSwapchain.GetSwapchainImages(
                device.Device,
                SwapchainKhr,
                ref imageCount,
                swapChainImagesPtr
            );
        }

        Format = surfaceFormat.Format;
        this.Extent = extent;
    }

    public void Dispose()
    {
        KhrSwapchain.DestroySwapchain(device.Device, SwapchainKhr, null);
    }

    public static bool HasSwapchainSupport(
        SurfaceWrapper surface,
        PhysicalDeviceWrapper physicalDevice
    )
    {
        return HasValidSurfaceFormat(surface, physicalDevice)
            && HasValidPresentMode(surface, physicalDevice);
    }

    public static bool HasValidSurfaceFormat(
        SurfaceWrapper surface,
        PhysicalDeviceWrapper physicalDevice
    )
    {
        try
        {
            GetBestSurfaceFormat(surface.GetPhysicalDeviceSurfaceFormats(physicalDevice));
            return true;
        }
        catch
        {
            return false;
        }
    }

    public static bool HasValidPresentMode(
        SurfaceWrapper surface,
        PhysicalDeviceWrapper physicalDevice
    )
    {
        try
        {
            GetBestPresentMode(surface.GetPhysicalDeviceSurfacePresentModes(physicalDevice));
            return true;
        }
        catch
        {
            return false;
        }
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
