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
        // TODO helper for creating and cleaning up temp files?
        string tempFilePath = Path.ChangeExtension(
            Path.GetTempFileName(),
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
        string outputFilePath = Path.GetTempFileName();
        Log.Value.LogTrace(
            "writing shader of type {ShaderType} source to: {TempFilePath}, compiling to Spir-V at output path: {OutputFilePath}",
            shaderType,
            tempFilePath,
            outputFilePath
        );

        try
        {
            File.WriteAllText(tempFilePath, source);

            Exec.Run("glslc", [tempFilePath, "-x", "glsl", "-o", outputFilePath]);

            return new ShaderModuleWrapper(vk, device, File.ReadAllBytes(outputFilePath));
        }
        finally
        {
            try
            {
                if (File.Exists(tempFilePath))
                {
                    File.Delete(tempFilePath);
                }
            }
            catch (Exception e)
            {
                Log.Value.LogError(
                    e,
                    "failed to clean up temporary shader source file at path {TempFilePath}",
                    tempFilePath
                );
            }

            try
            {
                if (File.Exists(outputFilePath))
                {
                    File.Delete(outputFilePath);
                }
            }
            catch (Exception e)
            {
                Log.Value.LogError(
                    e,
                    "failed to clean up temporary shader compilation output file at path {OutputFilePath}",
                    outputFilePath
                );
            }
        }
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
