using Silk.NET.Shaderc;

namespace Experiment.VulkanUtils;

public sealed unsafe class ShadercCompilerOptionsWrapper : IDisposable
{
    private readonly Shaderc shaderc;

    public readonly CompileOptions* CompileOptions;

    public ShadercCompilerOptionsWrapper(Shaderc shaderc)
    {
        this.shaderc = shaderc;
        CompileOptions = shaderc.CompileOptionsInitialize();
    }

    public void Dispose()
    {
        shaderc.CompileOptionsRelease(CompileOptions);
    }
}
