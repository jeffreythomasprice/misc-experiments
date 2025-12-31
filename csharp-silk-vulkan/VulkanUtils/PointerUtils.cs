using System.Text;
using Silk.NET.Core.Native;

namespace Experiment.VulkanUtils;

public static class PointerUtils
{
    public sealed class DisposableStringPointer(
        string input,
        NativeStringEncoding encoding = NativeStringEncoding.Ansi
    ) : IDisposable
    {
        public readonly string Value = input;
        public readonly nint Pointer = SilkMarshal.StringToPtr(input, encoding);

        public void Dispose()
        {
            SilkMarshal.Free(Pointer);
        }
    }

    public sealed class DisposableStringArrayPointer(
        IReadOnlyList<string> inputs,
        NativeStringEncoding encoding = NativeStringEncoding.Ansi
    ) : IDisposable
    {
        public readonly IReadOnlyList<string> Value = inputs;
        public readonly nint Pointer = SilkMarshal.StringArrayToPtr(inputs, encoding);

        public void Dispose()
        {
            SilkMarshal.Free(Pointer);
        }
    }
}
