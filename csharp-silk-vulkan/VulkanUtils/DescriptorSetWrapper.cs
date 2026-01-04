namespace Experiment.VulkanUtils;

using System.Runtime.CompilerServices;
using Microsoft.Extensions.Logging;
using Silk.NET.Vulkan;

public sealed unsafe class DescriptorSetWrapper : IDisposable
{
    private readonly ILogger log;

    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly DescriptorPoolWrapper descriptorPool;

    public readonly DescriptorSet DescriptorSet;

    public DescriptorSetWrapper(
        Vk vk,
        DeviceWrapper device,
        DescriptorPoolWrapper descriptorPool,
        DescriptorSetLayoutWrapper descriptorSetLayout
    )
    {
        log = LoggerUtils.Factory.Value.CreateLogger(GetType());

        this.vk = vk;
        this.device = device;
        this.descriptorPool = descriptorPool;

        log.LogDebug("allocating descriptor set");

        fixed (DescriptorSetLayout* layoutPtr = &descriptorSetLayout.DescriptorSetLayout)
        {
            var allocInfo = new DescriptorSetAllocateInfo()
            {
                SType = StructureType.DescriptorSetAllocateInfo,
                DescriptorPool = descriptorPool.DescriptorPool,
                DescriptorSetCount = 1,
                PSetLayouts = layoutPtr,
            };

            if (
                vk.AllocateDescriptorSets(device.Device, in allocInfo, out DescriptorSet)
                != Result.Success
            )
            {
                throw new Exception("failed to allocate descriptor set");
            }
        }
    }

    public void Dispose()
    {
        vk.FreeDescriptorSets(device.Device, descriptorPool.DescriptorPool, 1, in DescriptorSet);
    }

    public void UpdateDescriptorSet<T>(BufferWrapper<T> buffer, uint binding)
    {
        var bufferInfo = new DescriptorBufferInfo()
        {
            Buffer = buffer.Buffer,
            Offset = 0,
            Range = (ulong)Unsafe.SizeOf<UniformBufferObject>(),
        };

        var descriptorWrite = new WriteDescriptorSet()
        {
            SType = StructureType.WriteDescriptorSet,
            DstSet = DescriptorSet,
            DstBinding = binding,
            DstArrayElement = 0,
            DescriptorType = DescriptorType.UniformBuffer,
            DescriptorCount = 1,
            PBufferInfo = &bufferInfo,
        };

        vk.UpdateDescriptorSets(device.Device, 1, in descriptorWrite, 0, null);
    }
}
