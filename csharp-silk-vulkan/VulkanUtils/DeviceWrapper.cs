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

public sealed unsafe class DeviceWrapper : IDisposable
{
    private readonly Vk vk;
    public readonly Device Device;
    public readonly Queue GraphicsQueue;
    public readonly Queue PresentQueue;

    public DeviceWrapper(Vk vk, PhysicalDeviceWrapper physicalDevice, bool enableValidationLayers)
    {
        this.vk = vk;

        var graphicsQueueIndex = physicalDevice.AssertGraphicsQueueIndex();
        var presentQueueIndex = physicalDevice.AssertPresentQueueIndex();
        var uniqueQueueFamilies = new[] { graphicsQueueIndex, presentQueueIndex }
            .Distinct()
            .ToArray();

        using var mem = GlobalMemory.Allocate(
            uniqueQueueFamilies.Length * sizeof(DeviceQueueCreateInfo)
        );
        var queueCreateInfos = (DeviceQueueCreateInfo*)
            Unsafe.AsPointer(ref mem.GetPinnableReference());

        float queuePriority = 1.0f;
        for (int i = 0; i < uniqueQueueFamilies.Length; i++)
        {
            queueCreateInfos[i] = new()
            {
                SType = StructureType.DeviceQueueCreateInfo,
                QueueFamilyIndex = uniqueQueueFamilies[i],
                QueueCount = 1,
                PQueuePriorities = &queuePriority,
            };
        }

        var deviceFeatures = new PhysicalDeviceFeatures();

        using var extensions = new PointerUtils.DisposableStringArrayPointer(
            ExtensionsUtils.GetRequiredDeviceExtensions()
        );
        var createInfo = new DeviceCreateInfo()
        {
            SType = StructureType.DeviceCreateInfo,
            QueueCreateInfoCount = (uint)uniqueQueueFamilies.Length,
            PQueueCreateInfos = queueCreateInfos,
            PEnabledFeatures = &deviceFeatures,
            EnabledExtensionCount = (uint)extensions.Value.Count,
            PpEnabledExtensionNames = (byte**)extensions.Pointer,
        };

        using var layers = new PointerUtils.DisposableStringArrayPointer(
            DebugMessengerWrapper.REQUIRED_VALIDATION_LAYERS
        );
        if (enableValidationLayers)
        {
            createInfo.EnabledLayerCount = (uint)layers.Value.Count;
            createInfo.PpEnabledLayerNames = (byte**)layers.Pointer;
        }
        else
        {
            createInfo.EnabledLayerCount = 0;
        }

        if (
            vk.CreateDevice(physicalDevice.PhysicalDevice, in createInfo, null, out Device)
            != Result.Success
        )
        {
            throw new Exception("failed to create logical device");
        }

        vk.GetDeviceQueue(Device, graphicsQueueIndex, 0, out GraphicsQueue);
        vk.GetDeviceQueue(Device, presentQueueIndex, 0, out PresentQueue);
    }

    public void Dispose()
    {
        vk.DestroyDevice(Device, null);
    }
}
