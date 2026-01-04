namespace Experiment.VulkanUtils;

using System;
using System.Xml;
using Microsoft.Extensions.Logging;
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
        DeviceWrapper device,
        ShaderType shaderType,
        string source
    )
    {
        using var sourceFilePath = new TempFilePath(
            shaderType switch
            {
                ShaderType.Vertex => "vert",
                ShaderType.Fragment => "frag",
                _ => throw new ArgumentOutOfRangeException(
                    nameof(shaderType),
                    "unknown shader type"
                ),
            }
        );
        using var outputFilePath = new TempFilePath();
        Log.Value.LogTrace(
            "writing shader of type {ShaderType} source to: {TempFilePath}, compiling to Spir-V at output path: {OutputFilePath}",
            shaderType,
            sourceFilePath.Path,
            outputFilePath.Path
        );

        File.WriteAllText(sourceFilePath.Path, source);

        Exec.Run("glslc", [sourceFilePath.Path, "-x", "glsl", "-o", outputFilePath.Path]);

        return new ShaderModuleWrapper(vk, device, File.ReadAllBytes(outputFilePath.Path));
    }

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
