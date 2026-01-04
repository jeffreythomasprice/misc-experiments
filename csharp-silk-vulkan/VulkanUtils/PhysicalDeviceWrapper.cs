namespace Experiment.VulkanUtils;

using System.Runtime.InteropServices;
using Microsoft.Extensions.Logging;
using Silk.NET.Vulkan;

public unsafe class PhysicalDeviceWrapper
{
    private static readonly Lazy<ILogger> log = new(() =>
        LoggerUtils.Factory.Value.CreateLogger<PhysicalDeviceWrapper>()
    );

    private readonly Vk vk;

    public readonly PhysicalDevice PhysicalDevice;

    public readonly string DeviceName;
    public readonly PhysicalDeviceType DeviceType;
    public readonly uint? GraphicsQueueIndex;
    public readonly uint? PresentQueueIndex;

    public static PhysicalDeviceWrapper FindBest(Vk vk, Instance instance, SurfaceWrapper surface)
    {
        var devices = vk.GetPhysicalDevices(instance);
        var bestDevice = devices
            .Select(d =>
            {
                var result = new PhysicalDeviceWrapper(vk, surface, d);
                log.Value.LogDebug("Physical device: {Device}", result);
                return result;
            })
            .Where(d =>
                d.GraphicsQueueIndex.HasValue
                && d.PresentQueueIndex.HasValue
                && d.HasAllRequiredExtensions
                && SwapchainWrapper.HasSwapchainSupport(surface, d)
                && d.PhysicalDeviceFeatures.SamplerAnisotropy
            )
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

        log.Value.LogInformation("Selected device: {Device}", bestDevice);
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

    public uint FindMemoryType(uint typeFilter, MemoryPropertyFlags properties)
    {
        vk.GetPhysicalDeviceMemoryProperties(PhysicalDevice, out var memProperties);

        for (uint i = 0; i < memProperties.MemoryTypeCount; i++)
        {
            if (
                (typeFilter & (1 << (int)i)) != 0
                && (memProperties.MemoryTypes[(int)i].PropertyFlags & properties) == properties
            )
            {
                return i;
            }
        }

        throw new Exception("failed to find suitable memory type");
    }

    private bool HasAllRequiredExtensions =>
        Extensions.IsSupersetOf(ExtensionsUtils.GetRequiredDeviceExtensions());

    private ISet<string> Extensions
    {
        get
        {
            uint extentionsCount = 0;
            vk.EnumerateDeviceExtensionProperties(
                PhysicalDevice,
                (byte*)null,
                ref extentionsCount,
                null
            );

            var availableExtensions = new ExtensionProperties[extentionsCount];
            fixed (ExtensionProperties* availableExtensionsPtr = availableExtensions)
            {
                vk.EnumerateDeviceExtensionProperties(
                    PhysicalDevice,
                    (byte*)null,
                    ref extentionsCount,
                    availableExtensionsPtr
                );
            }

            return availableExtensions
                .Select(extension => Marshal.PtrToStringAnsi((IntPtr)extension.ExtensionName))
                .OfType<string>()
                .ToHashSet();
        }
    }

    private PhysicalDeviceFeatures PhysicalDeviceFeatures
    {
        get
        {
            vk.GetPhysicalDeviceFeatures(PhysicalDevice, out var result);
            return result;
        }
    }
}
