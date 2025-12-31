namespace Experiment.VulkanUtils;

using System.Reflection.Metadata;
using Microsoft.Extensions.Logging;
using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.KHR;

public sealed unsafe class SurfaceWrapper : IDisposable
{
    private readonly ILogger log;
    private readonly InstanceWrapper instance;
    public readonly KhrSurface KhrSurface;
    public readonly SurfaceKHR SurfaceKHR;

    public SurfaceWrapper(IVkSurface windowSurface, Vk vk, InstanceWrapper instance)
    {
        log = LoggerUtils.Factory.Value.CreateLogger(GetType());
        this.instance = instance;
        if (!vk.TryGetInstanceExtension<KhrSurface>(instance.Instance, out KhrSurface))
        {
            throw new NotSupportedException("KHR_surface extension not found.");
        }
        SurfaceKHR = windowSurface
            .Create<AllocationCallbacks>(instance.Instance.ToHandle(), null)
            .ToSurface();
    }

    public void Dispose()
    {
        KhrSurface.DestroySurface(instance.Instance, SurfaceKHR, null);
    }

    public SurfaceCapabilitiesKHR GetPhysicalDeviceSurfaceCapabilities(
        PhysicalDeviceWrapper physicalDevice
    )
    {
        // TODO check result
        KhrSurface.GetPhysicalDeviceSurfaceCapabilities(
            physicalDevice.PhysicalDevice,
            SurfaceKHR,
            out var result
        );
        log.LogDebug(
            "Physical device capabilities on this surface, MinImageCount={MinImageCount}, MaxImageCount={MaxImageCount}, CurrentExtent={CurrentExtentWidth}x{CurrentExtentHeight}, MinImageExtent={MinImageExtentWidth}x{MinImageExtentHeight}, MaxImageExtent={MaxImageExtentWidth}x{MaxImageExtentHeight}, MaxImageArrayLayers={MaxImageArrayLayers}, SupportedTransforms={SupportedTransforms}, CurrentTransform={CurrentTransform}, SupportedCompositeAlpha={SupportedCompositeAlpha}, SupportedUsageFlags={SupportedUsageFlags}",
            result.MinImageCount,
            result.MaxImageCount,
            result.CurrentExtent.Width,
            result.CurrentExtent.Height,
            result.MinImageExtent.Width,
            result.MinImageExtent.Height,
            result.MaxImageExtent.Width,
            result.MaxImageExtent.Height,
            result.MaxImageArrayLayers,
            result.SupportedTransforms,
            result.CurrentTransform,
            result.SupportedCompositeAlpha,
            result.SupportedUsageFlags
        );
        return result;
    }

    public SurfaceFormatKHR[] GetPhysicalDeviceSurfaceFormats(PhysicalDeviceWrapper physicalDevice)
    {
        uint count = 0;
        KhrSurface.GetPhysicalDeviceSurfaceFormats(
            physicalDevice.PhysicalDevice,
            SurfaceKHR,
            ref count,
            null
        );

        log.LogDebug("physical device surface format count: {Count}", count);

        if (count == 0)
        {
            return [];
        }

        var results = new SurfaceFormatKHR[count];
        fixed (SurfaceFormatKHR* formatsPtr = results)
        {
            KhrSurface.GetPhysicalDeviceSurfaceFormats(
                physicalDevice.PhysicalDevice,
                SurfaceKHR,
                ref count,
                formatsPtr
            );
        }

        log.LogDebug($"physical device surface formats:");
        foreach (var (x, i) in results.Select((x, i) => (x, i)))
        {
            log.LogDebug(
                "    Format[{Index}]: {Format}, ColorSpace: {ColorSpace}",
                i,
                x.Format,
                x.ColorSpace
            );
        }

        return results;
    }

    public PresentModeKHR[] GetPhysicalDeviceSurfacePresentModes(
        PhysicalDeviceWrapper physicalDevice
    )
    {
        uint count = 0;
        KhrSurface.GetPhysicalDeviceSurfacePresentModes(
            physicalDevice.PhysicalDevice,
            SurfaceKHR,
            ref count,
            null
        );

        log.LogDebug("physical device surface present mode count: {Count}", count);

        if (count == 0)
        {
            return [];
        }

        var results = new PresentModeKHR[count];
        fixed (PresentModeKHR* formatsPtr = results)
        {
            KhrSurface.GetPhysicalDeviceSurfacePresentModes(
                physicalDevice.PhysicalDevice,
                SurfaceKHR,
                ref count,
                formatsPtr
            );
        }

        log.LogDebug("physical device surface present modes: [{PresentModes}]", results);

        return results;
    }
}
