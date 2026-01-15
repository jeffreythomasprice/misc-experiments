namespace Experiment.Engine;

using System.ComponentModel;
using Experiment.VulkanUtils;
using Silk.NET.Vulkan;

public sealed unsafe class Uniforms : IDisposable
{
    public class BindingBase : IDisposable
    {
        protected readonly Uniforms Parent;
        protected readonly uint Binding;

        internal BindingBase(Uniforms parent, uint binding)
        {
            this.Parent = parent;
            this.Binding = binding;
        }

        public virtual void Dispose() { }
    }

    public sealed class BufferBinding<T> : BindingBase
    {
        private readonly BufferWrapper<T> buffer;

        internal BufferBinding(Uniforms parent, uint binding)
            : base(parent, binding)
        {
            buffer = new(
                parent.vk,
                parent.physicalDevice,
                parent.device,
                1,
                BufferUsageFlags.UniformBufferBit
            );
        }

        public override void Dispose()
        {
            buffer.Dispose();
        }

        public T Value
        {
            get
            {
                T? result = default;
                buffer.GetWritableSpanToBufferData(span =>
                {
                    result = span[0];
                });
                return result!;
            }
            set
            {
                buffer.CopyDataToBuffer([value]);
                Parent.DescriptorSet.UpdateDescriptorSet<T>(buffer, Binding);
            }
        }
    }

    public sealed class TextureBinding : BindingBase
    {
        internal TextureBinding(Uniforms parent, uint binding)
            : base(parent, binding) { }

        public TextureImageWrapper? Value
        {
            get;
            set
            {
                field = value;
                if (value is not null)
                {
                    Parent.DescriptorSet.UpdateDescriptorSet(value, Binding);
                }
            }
        }
    }

    private readonly Vk vk;
    private readonly PhysicalDeviceWrapper physicalDevice;
    private readonly DeviceWrapper device;

    private readonly IReadOnlyList<DescriptorSetLayoutBinding> bindings;

    private readonly DescriptorPoolWrapper descriptorPool;

    public readonly DescriptorSetLayoutWrapper DescriptorSetLayout;
    public readonly DescriptorSetWrapper DescriptorSet;

    public Uniforms(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        DescriptorSetLayoutBinding[] bindings
    )
    {
        this.vk = vk;
        this.physicalDevice = physicalDevice;
        this.device = device;

        this.bindings = [.. bindings];

        /*
        TODO separate the parts of uniforms out into different pieces:
        - we sometimes need an array of different descriptor sets, these should share a pool
        - we want the layout to be separate entirely
        */

        descriptorPool = new(
            vk,
            device,
            [
                .. bindings
                    .GroupBy(b => b.DescriptorType)
                    .Select(x => new DescriptorPoolSize
                    {
                        Type = x.Key,
                        DescriptorCount = (uint)x.Sum(b => b.DescriptorCount),
                    }),
            ],
            1
        );

        DescriptorSetLayout = new DescriptorSetLayoutWrapper(vk, device, bindings);
        DescriptorSet = new DescriptorSetWrapper(vk, device, descriptorPool, DescriptorSetLayout);

        foreach (var binding in bindings)
        {
            if (binding.DescriptorType == DescriptorType.UniformBuffer)
            {
                // TODO make a uniform helper here
            }
            else if (binding.DescriptorType == DescriptorType.CombinedImageSampler)
            {
                // TODO make a texture helper here
            }
            else
            {
                throw new NotSupportedException(
                    $"Descriptor type {binding.DescriptorType} not supported"
                );
            }
        }
    }

    public void Dispose()
    {
        DescriptorSet.Dispose();
        DescriptorSetLayout.Dispose();
        descriptorPool.Dispose();
    }

    public BufferBinding<T> GetBufferBinding<T>(uint binding)
    {
        AssertBindingIsType(binding, DescriptorType.UniformBuffer);
        return new BufferBinding<T>(this, binding);
    }

    public TextureBinding GetTextureBinding(uint binding)
    {
        AssertBindingIsType(binding, DescriptorType.CombinedImageSampler);
        return new TextureBinding(this, binding);
    }

    private void AssertBindingIsType(uint binding, DescriptorType expectedType)
    {
        DescriptorSetLayoutBinding? layoutBinding = bindings.FirstOrDefault(b =>
            b.Binding == binding
        );
        if (layoutBinding is null)
        {
            throw new InvalidOperationException($"Binding {binding} not found");
        }
        if (layoutBinding.Value.DescriptorType != expectedType)
        {
            throw new InvalidOperationException(
                $"Binding {binding} is of type {layoutBinding.Value.DescriptorType}, expected {expectedType}"
            );
        }
    }
}
