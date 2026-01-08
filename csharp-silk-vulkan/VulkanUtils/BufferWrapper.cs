namespace Experiment.VulkanUtils;

using System;
using System.Runtime.CompilerServices;
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

        (Buffer, BufferMemory) = Init(vk, physicalDevice, device, SizeInBytes, usage);

        CopyDataToBuffer(data);
    }

    public BufferWrapper(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        int count,
        BufferUsageFlags usage
    )
    {
        if (count <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(count), "must be positive");
        }

        this.vk = vk;
        this.device = device;

        SizeInBytes = (UInt64)(count * Marshal.SizeOf<T>());
        Count = count;

        (Buffer, BufferMemory) = Init(vk, physicalDevice, device, SizeInBytes, usage);
    }

    public BufferWrapper(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        int count,
        BufferUsageFlags usage,
        Action<Span<T>> initialDataCallback
    )
        : this(vk, physicalDevice, device, count, usage)
    {
        GetWritableSpanToBufferData(initialDataCallback);
    }

    public void Dispose()
    {
        vk.FreeMemory(device.Device, BufferMemory, null);
        vk.DestroyBuffer(device.Device, Buffer, null);
    }

    public void CopyDataToBuffer(ReadOnlySpan<T> data)
    {
        CopyDataToBuffer(data, 0);
    }

    /// <param name="data"></param>
    /// <param name="offset">as an index, not a byte offset</param>
    public void CopyDataToBuffer(ReadOnlySpan<T> data, int offset)
    {
        var stride = (UInt64)Marshal.SizeOf<T>();
        void* dataPtr;
        vk.MapMemory(
            device.Device,
            BufferMemory,
            (UInt64)offset * stride,
            (UInt64)data.Length * stride,
            0,
            &dataPtr
        );
        try
        {
            data.CopyTo(new Span<T>(dataPtr, data.Length));
        }
        finally
        {
            vk.UnmapMemory(device.Device, BufferMemory);
        }
    }

    public void GetWritableSpanToBufferData(Action<Span<T>> f)
    {
        GetWritableSpanToBufferData(f, 0, SizeInBytes);
    }

    public void GetWritableSpanToBufferData(Action<Span<T>> f, UInt64 offset, UInt64 sizeInBytes)
    {
        void* dataPtr;
        vk.MapMemory(device.Device, BufferMemory, offset, sizeInBytes, 0, &dataPtr);
        try
        {
            f(new Span<T>(dataPtr, (int)sizeInBytes / Unsafe.SizeOf<T>()));
        }
        finally
        {
            vk.UnmapMemory(device.Device, BufferMemory);
        }
    }

    private static (Silk.NET.Vulkan.Buffer, DeviceMemory) Init(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        UInt64 sizeInBytes,
        BufferUsageFlags usage
    )
    {
        var bufferInfo = new BufferCreateInfo()
        {
            SType = StructureType.BufferCreateInfo,
            Size = sizeInBytes,
            Usage = usage,
            SharingMode = SharingMode.Exclusive,
        };

        if (vk.CreateBuffer(device.Device, in bufferInfo, null, out var buffer) != Result.Success)
        {
            throw new Exception("failed to create buffer");
        }

        vk.GetBufferMemoryRequirements(device.Device, buffer, out var memRequirements);

        var allocInfo = new MemoryAllocateInfo()
        {
            SType = StructureType.MemoryAllocateInfo,
            AllocationSize = memRequirements.Size,
            MemoryTypeIndex = physicalDevice.FindMemoryType(
                memRequirements.MemoryTypeBits,
                MemoryPropertyFlags.HostVisibleBit | MemoryPropertyFlags.HostCoherentBit
            ),
        };

        if (
            vk.AllocateMemory(device.Device, in allocInfo, null, out var bufferMemory)
            != Result.Success
        )
        {
            throw new Exception("failed to allocate buffer memory");
        }

        vk.BindBufferMemory(device.Device, buffer, bufferMemory, 0);

        return (buffer, bufferMemory);
    }
}
