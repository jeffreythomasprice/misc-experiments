namespace Experiment.VulkanUtils;

using Silk.NET.Vulkan;

public sealed unsafe class DescriptorPoolWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;

    public readonly DescriptorPool DescriptorPool;

    public DescriptorPoolWrapper(
        Vk vk,
        DeviceWrapper device,
        DescriptorPoolSize[] poolSizes,
        uint maxSets
    )
    {
        this.vk = vk;
        this.device = device;

        // TODO logging

        fixed (DescriptorPoolSize* poolSizesPtr = poolSizes)
        {
            var poolInfo = new DescriptorPoolCreateInfo()
            {
                SType = StructureType.DescriptorPoolCreateInfo,
                PoolSizeCount = (uint)poolSizes.Length,
                PPoolSizes = poolSizesPtr,
                MaxSets = maxSets,
            };

            if (
                vk.CreateDescriptorPool(device.Device, in poolInfo, null, out DescriptorPool)
                != Result.Success
            )
            {
                throw new Exception("failed to create descriptor pool");
            }
        }
    }

    public void Dispose()
    {
        vk.DestroyDescriptorPool(device.Device, DescriptorPool, null);
    }
}
