namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class ShaderModuleWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    public readonly ShaderModule ShaderModule;

    public ShaderModuleWrapper(Vk vk, DeviceWrapper device, byte[] spirvBytes)
    {
        this.vk = vk;
        this.device = device;

        var createInfo = new ShaderModuleCreateInfo()
        {
            SType = StructureType.ShaderModuleCreateInfo,
            CodeSize = (nuint)spirvBytes.Length,
        };

        fixed (byte* spirvBytesPtr = spirvBytes)
        {
            createInfo.PCode = (uint*)spirvBytesPtr;

            if (
                vk.CreateShaderModule(device.Device, in createInfo, null, out ShaderModule)
                != Result.Success
            )
            {
                throw new Exception("failed to create shader module");
            }
        }
    }

    public void Dispose()
    {
        vk.DestroyShaderModule(device.Device, ShaderModule, null);
    }
}
