namespace Experiment.VulkanUtils;

using System;
using System.Runtime.InteropServices;
using Silk.NET.Vulkan;

public sealed unsafe class BufferWrapper<T> : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    public readonly UInt64 SizeInBytes;
    public readonly int Count;
    public readonly Silk.NET.Vulkan.Buffer Buffer;
    public readonly DeviceMemory BufferMemory;

    /*
    TODO support copying between two buffers
    this should let us update a buffer in another thread and copy to the display buffer before rendering
    https://github.com/dfkeenan/SilkVulkanTutorial/blob/main/Source/20_StagingBuffer/Program.cs
    */

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

        SizeInBytes = (UInt64)(data.Length * Marshal.SizeOf<T>());
        Count = data.Length;

        var bufferInfo = new BufferCreateInfo()
        {
            SType = StructureType.BufferCreateInfo,
            Size = SizeInBytes,
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

        CopyDataToBuffer(data);
    }

    public void Dispose()
    {
        vk.FreeMemory(device.Device, BufferMemory, null);
        vk.DestroyBuffer(device.Device, Buffer, null);
    }

    public void CopyDataToBuffer(ReadOnlySpan<T> data)
    {
        vk.BindBufferMemory(device.Device, Buffer, BufferMemory, 0);

        void* dataPtr;
        vk.MapMemory(device.Device, BufferMemory, 0, SizeInBytes, 0, &dataPtr);
        try
        {
            data.CopyTo(new Span<T>(dataPtr, data.Length));
        }
        finally
        {
            vk.UnmapMemory(device.Device, BufferMemory);
        }
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
