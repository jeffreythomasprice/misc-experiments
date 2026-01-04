namespace Experiment.VulkanUtils;

using Microsoft.Extensions.Logging;
using Silk.NET.Vulkan;

public sealed unsafe class DescriptorPoolWrapper : IDisposable
{
    private readonly ILogger log;

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
        log = LoggerUtils.Factory.Value.CreateLogger(GetType());

        this.vk = vk;
        this.device = device;

        log.LogDebug(
            "creating descriptor pool with {PoolSizeCount} pool sizes, max sets {MaxSets}",
            poolSizes.Length,
            maxSets
        );
        foreach (var poolSize in poolSizes)
        {
            log.LogDebug(
                "pool size: Type={Type}, DescriptorCount={DescriptorCount}",
                poolSize.Type,
                poolSize.DescriptorCount
            );
        }

        fixed (DescriptorPoolSize* poolSizesPtr = poolSizes)
        {
            var poolInfo = new DescriptorPoolCreateInfo()
            {
                SType = StructureType.DescriptorPoolCreateInfo,
                PoolSizeCount = (uint)poolSizes.Length,
                PPoolSizes = poolSizesPtr,
                MaxSets = maxSets,
                Flags = DescriptorPoolCreateFlags.FreeDescriptorSetBit,
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
