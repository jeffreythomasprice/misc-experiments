namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class CommandPoolWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    public readonly CommandPool CommandPool;

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
            vk.CreateCommandPool(device.Device, in poolInfo, null, out CommandPool)
            != Result.Success
        )
        {
            throw new Exception("failed to create command pool!");
        }
    }

    public void Dispose()
    {
        vk.DestroyCommandPool(device.Device, CommandPool, null);
    }
}
