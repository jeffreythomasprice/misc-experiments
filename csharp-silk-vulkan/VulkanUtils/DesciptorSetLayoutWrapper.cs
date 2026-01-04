namespace Experiment.VulkanUtils;

using Microsoft.Extensions.Logging;
using Silk.NET.Vulkan;

public sealed unsafe class DescriptorSetLayoutWrapper : IDisposable
{
    private readonly ILogger log;

    private readonly Vk vk;
    private readonly DeviceWrapper device;

    public readonly DescriptorSetLayout DescriptorSetLayout;

    public DescriptorSetLayoutWrapper(
        Vk vk,
        DeviceWrapper device,
        DescriptorSetLayoutBinding[] bindings
    )
    {
        log = LoggerUtils.Factory.Value.CreateLogger(GetType());

        this.vk = vk;
        this.device = device;

        log.LogDebug(
            "creating descriptor set layout with {BindingCount} bindings",
            bindings.Length
        );
        foreach (var binding in bindings)
        {
            log.LogDebug(
                "binding {Binding}: Type={DescriptorType}, Count={DescriptorCount}, StageFlags={StageFlags}",
                binding.Binding,
                binding.DescriptorType,
                binding.DescriptorCount,
                binding.StageFlags
            );
        }

        fixed (DescriptorSetLayoutBinding* bindingsPtr = bindings)
        {
            var layoutInfo = new DescriptorSetLayoutCreateInfo()
            {
                SType = StructureType.DescriptorSetLayoutCreateInfo,
                BindingCount = (uint)bindings.Length,
                PBindings = bindingsPtr,
            };

            if (
                vk.CreateDescriptorSetLayout(
                    device.Device,
                    in layoutInfo,
                    null,
                    out DescriptorSetLayout
                ) != Result.Success
            )
            {
                throw new Exception("failed to create descriptor set layout");
            }
        }
    }

    public void Dispose()
    {
        vk.DestroyDescriptorSetLayout(device.Device, DescriptorSetLayout, null);
    }
}
