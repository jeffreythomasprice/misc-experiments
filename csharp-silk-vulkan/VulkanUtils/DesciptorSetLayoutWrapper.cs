namespace Experiment.VulkanUtils;

using Silk.NET.Vulkan;

public sealed unsafe class DescriptorSetLayoutWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;

    public readonly DescriptorSetLayout DescriptorSetLayout;

    public DescriptorSetLayoutWrapper(
        Vk vk,
        DeviceWrapper device,
        DescriptorSetLayoutBinding[] bindings
    )
    {
        this.vk = vk;
        this.device = device;

        // TODO logging

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
