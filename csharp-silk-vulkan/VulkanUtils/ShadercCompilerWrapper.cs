namespace Experiment.VulkanUtils;

using Microsoft.Extensions.Logging;
using Silk.NET.Shaderc;

public sealed unsafe class ShadercCompilerWrapper : IDisposable
{
    private static readonly Lazy<ILogger> Log = new(() =>
        LoggerUtils.Factory.Value.CreateLogger<ShadercCompilerWrapper>()
    );

    private readonly Shaderc shaderc;
    private readonly Compiler* compiler;

    public ShadercCompilerWrapper(Shaderc shaderc)
    {
        this.shaderc = shaderc;
        compiler = shaderc.CompilerInitialize();
    }

    public void Dispose()
    {
        shaderc.CompilerRelease(compiler);
    }

    public byte[] CompileGlslSource(
        string source,
        ShaderKind kind,
        ShadercCompilerOptionsWrapper options
    )
    {
        var compilationResult = shaderc.CompileIntoSpv(
            compiler,
            source,
            (nuint)source.Length,
            kind,
            // input file name unused? or at least unused in my limited testing
            "",
            // entry point is used only for HLSL compilation, for GLSL it is always "main"
            // https://github.com/google/shaderc/issues/1465
            "",
            options.CompileOptions
        );
        try
        {
            var status = shaderc.ResultGetCompilationStatus(compilationResult);
            Log.Value.LogTrace(
                "compiling glsl shader using shaderc, shader of kind {Kind}, status: {Status}",
                kind,
                status
            );
            if (status != CompilationStatus.Success)
            {
                var numErrors = shaderc.ResultGetNumErrors(compilationResult);
                var numWarnings = shaderc.ResultGetNumWarnings(compilationResult);
                var errorMessage = shaderc.ResultGetErrorMessageS(compilationResult);
                Log.Value.LogError(
                    "shader of kind {Kind} compilation failed, status: {Status}, {NumErrors} errors and {NumWarnings} warnings, error message: {ErrorMessage}",
                    kind,
                    status,
                    numErrors,
                    numWarnings,
                    errorMessage
                );
                throw new Exception("shader compilation failed");
            }
            return new Span<byte>(
                shaderc.ResultGetBytes(compilationResult),
                (int)shaderc.ResultGetLength(compilationResult)
            ).ToArray();
        }
        finally
        {
            shaderc.ResultRelease(compilationResult);
        }
    }
}
