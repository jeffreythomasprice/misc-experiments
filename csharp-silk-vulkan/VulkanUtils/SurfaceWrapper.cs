namespace Experiment.VulkanUtils;

using System.Reflection.Metadata;
using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.KHR;

public sealed unsafe class SurfaceWrapper : IDisposable
{
    private readonly InstanceWrapper instance;
    public readonly KhrSurface KhrSurface;
    public readonly SurfaceKHR SurfaceKHR;

    public SurfaceWrapper(IVkSurface windowSurface, Vk vk, InstanceWrapper instance)
    {
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
        // TODO proper logging
        Console.WriteLine($"physical device capabilities on this surface:");
        Console.WriteLine($"    MinImageCount: {result.MinImageCount}");
        Console.WriteLine($"    MaxImageCount: {result.MaxImageCount}");
        Console.WriteLine(
            $"    CurrentExtent: {result.CurrentExtent.Width}x{result.CurrentExtent.Height}"
        );
        Console.WriteLine(
            $"    MinImageExtent: {result.MinImageExtent.Width}x{result.MinImageExtent.Height}"
        );
        Console.WriteLine(
            $"    MaxImageExtent: {result.MaxImageExtent.Width}x{result.MaxImageExtent.Height}"
        );
        Console.WriteLine($"    MaxImageArrayLayers: {result.MaxImageArrayLayers}");
        Console.WriteLine($"    SupportedTransforms: {result.SupportedTransforms}");
        Console.WriteLine($"    CurrentTransform: {result.CurrentTransform}");
        Console.WriteLine($"    SupportedCompositeAlpha: {result.SupportedCompositeAlpha}");
        Console.WriteLine($"    SupportedUsageFlags: {result.SupportedUsageFlags}");
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

        // TODO proper logging
        Console.WriteLine($"physical device surface format count: {count}");

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

        // TODO proper logging
        Console.WriteLine($"physical device surface formats:");
        foreach (var (x, i) in results.Select((x, i) => (x, i)))
        {
            Console.WriteLine($"    Format[{i}]: {x.Format}, ColorSpace: {x.ColorSpace}");
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

        // TODO proper logging
        Console.WriteLine($"physical device surface present mode count: {count}");

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

        // TODO proper logging
        Console.WriteLine($"physical device surface present modes: [{string.Join(", ", results)}]");

        return results;
    }
}
