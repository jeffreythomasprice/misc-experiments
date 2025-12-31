namespace Experiment.VulkanUtils;

using System.Runtime.InteropServices;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.KHR;

public unsafe class PhysicalDeviceWrapper
{
    private readonly Vk vk;

    public readonly PhysicalDevice PhysicalDevice;

    public readonly string DeviceName;
    public readonly PhysicalDeviceType DeviceType;
    public readonly uint? GraphicsQueueIndex;
    public readonly uint? PresentQueueIndex;

    public static PhysicalDeviceWrapper FindBest(Vk vk, Instance instance, SurfaceWrapper surface)
    {
        // TODO proper logging

        var devices = vk.GetPhysicalDevices(instance);
        var bestDevice = devices
            .Select(d =>
            {
                var result = new PhysicalDeviceWrapper(vk, surface, d);
                Console.WriteLine($"physical device: {result}");
                return result;
            })
            .Where(d => d.GraphicsQueueIndex.HasValue)
            .OrderBy(d =>
                d.DeviceType switch
                {
                    PhysicalDeviceType.DiscreteGpu => 0,
                    PhysicalDeviceType.IntegratedGpu => 1,
                    PhysicalDeviceType.Cpu => 2,
                    _ => 3,
                }
            )
            .First();

        Console.WriteLine($"Selected device: {bestDevice})");
        return bestDevice;
    }

    public PhysicalDeviceWrapper(Vk vk, SurfaceWrapper surface, PhysicalDevice physicalDevice)
    {
        this.vk = vk;
        this.PhysicalDevice = physicalDevice;

        var properties = vk.GetPhysicalDeviceProperties(physicalDevice);

        DeviceName =
            Marshal.PtrToStringAnsi((nint)properties.DeviceName)
            ?? throw new NullReferenceException("failed to get device name");
        DeviceType = properties.DeviceType;

        uint queueFamilyPropertyCount = 0;
        vk.GetPhysicalDeviceQueueFamilyProperties(
            physicalDevice,
            ref queueFamilyPropertyCount,
            null
        );
        var queueFamilyProperties = new QueueFamilyProperties[queueFamilyPropertyCount];
        fixed (QueueFamilyProperties* ptr = queueFamilyProperties)
        {
            vk.GetPhysicalDeviceQueueFamilyProperties(
                physicalDevice,
                ref queueFamilyPropertyCount,
                ptr
            );
        }

        foreach (var (property, index) in queueFamilyProperties.Select((x, i) => (x, i)))
        {
            if (!GraphicsQueueIndex.HasValue && property.QueueFlags.HasFlag(QueueFlags.GraphicsBit))
            {
                GraphicsQueueIndex = (uint)index;
            }
            if (!PresentQueueIndex.HasValue)
            {
                surface.KhrSurface.GetPhysicalDeviceSurfaceSupport(
                    physicalDevice,
                    (uint)index,
                    surface.SurfaceKHR,
                    out var presentSupport
                );
                if (presentSupport)
                {
                    PresentQueueIndex = (uint)index;
                }
            }
        }
    }

    public override string ToString()
    {
        return $"PhysicalDevice(Name={DeviceName}, Type={DeviceType}, GraphicsQueueIndex={GraphicsQueueIndex})";
    }

    public uint AssertGraphicsQueueIndex()
    {
        if (!GraphicsQueueIndex.HasValue)
        {
            throw new Exception("selected physical device has no graphics queue");
        }
        return GraphicsQueueIndex.Value;
    }

    public uint AssertPresentQueueIndex()
    {
        if (!PresentQueueIndex.HasValue)
        {
            throw new Exception("selected physical device has no present queue");
        }
        return PresentQueueIndex.Value;
    }
}
