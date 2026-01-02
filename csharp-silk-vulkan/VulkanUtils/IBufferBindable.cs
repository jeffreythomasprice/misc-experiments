namespace Experiment.VulkanUtils;

using Silk.NET.Vulkan;

public interface IBufferBindable
{
    public static abstract VertexInputBindingDescription BindingDescription { get; }
    public static abstract VertexInputAttributeDescription[] AttributeDescriptions { get; }
}
