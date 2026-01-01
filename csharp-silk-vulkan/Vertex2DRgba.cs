namespace Experiment;

using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Silk.NET.Maths;
using Silk.NET.Vulkan;

public readonly struct Vertex2DRgba(Vector2D<float> position, Vector4D<float> color)
{
    public readonly Vector2D<float> Position = position;
    public readonly Vector4D<float> Color = color;

    public static VertexInputBindingDescription GetBindingDescription() =>
        new()
        {
            Binding = 0,
            Stride = (uint)Unsafe.SizeOf<Vertex2DRgba>(),
            InputRate = VertexInputRate.Vertex,
        };

    public static VertexInputAttributeDescription[] GetAttributeDescriptions() =>
        [
            new VertexInputAttributeDescription()
            {
                Binding = 0,
                Location = 0,
                Format = Format.R32G32Sfloat,
                Offset = (uint)Marshal.OffsetOf<Vertex2DRgba>(nameof(Position)),
            },
            new VertexInputAttributeDescription()
            {
                Binding = 0,
                Location = 1,
                Format = Format.R32G32B32A32Sfloat,
                Offset = (uint)Marshal.OffsetOf<Vertex2DRgba>(nameof(Color)),
            },
        ];
}
