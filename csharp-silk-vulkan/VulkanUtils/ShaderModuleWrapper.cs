namespace Experiment.VulkanUtils;

using System;
using System.Xml;
using Microsoft.Extensions.Logging;
using Silk.NET.Shaderc;
using Silk.NET.Vulkan;

public sealed unsafe class ShaderModuleWrapper : IDisposable
{
    public enum ShaderType
    {
        Vertex,
        Fragment,
    }

    private static readonly Lazy<ILogger> Log = new(() =>
        LoggerUtils.Factory.Value.CreateLogger<ShaderModuleWrapper>()
    );

    private readonly Vk vk;
    private readonly DeviceWrapper device;
    public readonly ShaderModule ShaderModule;

    public static ShaderModuleWrapper FromGlslSource(
        Vk vk,
        Shaderc shaderc,
        DeviceWrapper device,
        ShaderType shaderType,
        string source
    )
    {
        using var compiler = new ShadercCompilerWrapper(shaderc);
        using var compileOptions = new ShadercCompilerOptionsWrapper(shaderc);
        return new ShaderModuleWrapper(
            vk,
            device,
            compiler.CompileGlslSource(
                source,
                shaderType switch
                {
                    ShaderType.Vertex => ShaderKind.VertexShader,
                    ShaderType.Fragment => ShaderKind.FragmentShader,
                    _ => throw new ArgumentOutOfRangeException(
                        nameof(shaderType),
                        "unknown shader type"
                    ),
                },
                compileOptions
            )
        );
    }

    public ShaderModuleWrapper(Vk vk, DeviceWrapper device, byte[] bytes)
    {
        this.vk = vk;
        this.device = device;

        var createInfo = new ShaderModuleCreateInfo()
        {
            SType = StructureType.ShaderModuleCreateInfo,
            CodeSize = (nuint)bytes.Length,
        };

        fixed (byte* bytesPtr = bytes)
        {
            createInfo.PCode = (uint*)bytesPtr;

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
