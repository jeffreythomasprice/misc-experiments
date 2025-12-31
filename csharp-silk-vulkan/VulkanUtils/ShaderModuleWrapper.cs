namespace Experiment.VulkanUtils;

using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using Experiment.VulkanUtils;
using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Core.Native;
using Silk.NET.Maths;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.EXT;
using Silk.NET.Vulkan.Extensions.KHR;
using Silk.NET.Windowing;

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
                // TODO can we get compile error log?
                throw new Exception("failed to create shader module");
            }
        }
    }

    public void Dispose()
    {
        vk.DestroyShaderModule(device.Device, ShaderModule, null);
    }
}
