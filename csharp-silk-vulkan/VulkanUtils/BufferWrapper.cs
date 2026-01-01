namespace Experiment.VulkanUtils;

using System;
using System.Runtime.InteropServices;
using Silk.NET.Vulkan;

public sealed unsafe class BufferWrapper<T> : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    public readonly Silk.NET.Vulkan.Buffer Buffer;
    public readonly DeviceMemory BufferMemory;
    public readonly int Count;

    public BufferWrapper(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        ReadOnlySpan<T> data,
        BufferUsageFlags usage
    )
    {
        this.vk = vk;
        this.device = device;

        var bufferInfo = new BufferCreateInfo()
        {
            SType = StructureType.BufferCreateInfo,
            Size = (ulong)(data.Length * Marshal.SizeOf<T>()),
            Usage = usage,
            SharingMode = SharingMode.Exclusive,
        };

        if (vk.CreateBuffer(device.Device, in bufferInfo, null, out Buffer) != Result.Success)
        {
            throw new Exception("failed to create buffer");
        }

        vk.GetBufferMemoryRequirements(device.Device, Buffer, out var memRequirements);

        var allocInfo = new MemoryAllocateInfo()
        {
            SType = StructureType.MemoryAllocateInfo,
            AllocationSize = memRequirements.Size,
            MemoryTypeIndex = FindMemoryType(
                vk,
                physicalDevice,
                memRequirements.MemoryTypeBits,
                MemoryPropertyFlags.HostVisibleBit | MemoryPropertyFlags.HostCoherentBit
            ),
        };

        if (
            vk.AllocateMemory(device.Device, in allocInfo, null, out BufferMemory) != Result.Success
        )
        {
            throw new Exception("failed to allocate buffer memory");
        }

        vk.BindBufferMemory(device.Device, Buffer, BufferMemory, 0);

        void* dataPtr;
        vk.MapMemory(device.Device, BufferMemory, 0, bufferInfo.Size, 0, &dataPtr);
        try
        {
            data.CopyTo(new Span<T>(dataPtr, data.Length));
        }
        finally
        {
            vk.UnmapMemory(device.Device, BufferMemory);
        }

        Count = data.Length;
    }

    public void Dispose()
    {
        vk.FreeMemory(device.Device, BufferMemory, null);
        vk.DestroyBuffer(device.Device, Buffer, null);
    }

    private static uint FindMemoryType(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        uint typeFilter,
        MemoryPropertyFlags properties
    )
    {
        vk.GetPhysicalDeviceMemoryProperties(physicalDevice.PhysicalDevice, out var memProperties);

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
}
