namespace Experiment.VulkanUtils;

using System;
using System.Reflection.Metadata;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using Experiment.VulkanUtils;
using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Core.Native;
using Silk.NET.Maths;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.EXT;
using Silk.NET.Vulkan.Extensions.KHR;
using Silk.NET.Windowing;

public sealed unsafe class CommandPoolWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly CommandPool commandPool;

    public CommandPoolWrapper(Vk vk, PhysicalDeviceWrapper physicalDevice, DeviceWrapper device)
    {
        this.vk = vk;
        this.device = device;

        var poolInfo = new CommandPoolCreateInfo()
        {
            SType = StructureType.CommandPoolCreateInfo,
            QueueFamilyIndex = physicalDevice.AssertGraphicsQueueIndex(),
        };

        if (
            vk.CreateCommandPool(device.Device, in poolInfo, null, out commandPool)
            != Result.Success
        )
        {
            throw new Exception("failed to create command pool!");
        }
    }

    public void Dispose()
    {
        vk.DestroyCommandPool(device.Device, commandPool, null);
    }
}
