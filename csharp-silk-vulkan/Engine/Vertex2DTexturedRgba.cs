namespace Experiment.Engine;

using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Experiment.VulkanUtils;
using Silk.NET.Maths;
using Silk.NET.Vulkan;

public readonly struct Vertex2DTexturedRgba(
    Vector2D<float> position,
    Vector2D<float> textureCoordinate,
    Vector4D<float> color
) : IBufferBindable
{
    public const uint POSITION_LOCATION = 0;
    public const uint TEXTURE_COORDINATE_LOCATION = 1;
    public const uint COLOR_LOCATION = 2;

    public readonly Vector2D<float> Position = position;
    public readonly Vector2D<float> TextureCoordinate = textureCoordinate;
    public readonly Vector4D<float> Color = color;

    public static VertexInputBindingDescription BindingDescription =>
        new()
        {
            Binding = 0,
            Stride = (uint)Unsafe.SizeOf<Vertex2DTexturedRgba>(),
            InputRate = VertexInputRate.Vertex,
        };

    public static VertexInputAttributeDescription[] AttributeDescriptions =>
        [
            new()
            {
                Binding = 0,
                Location = POSITION_LOCATION,
                Format = Format.R32G32Sfloat,
                Offset = (uint)Marshal.OffsetOf<Vertex2DTexturedRgba>(nameof(Position)),
            },
            new()
            {
                Binding = 0,
                Location = TEXTURE_COORDINATE_LOCATION,
                Format = Format.R32G32Sfloat,
                Offset = (uint)Marshal.OffsetOf<Vertex2DTexturedRgba>(nameof(TextureCoordinate)),
            },
            new()
            {
                Binding = 0,
                Location = COLOR_LOCATION,
                Format = Format.R32G32B32A32Sfloat,
                Offset = (uint)Marshal.OffsetOf<Vertex2DTexturedRgba>(nameof(Color)),
            },
        ];
}
