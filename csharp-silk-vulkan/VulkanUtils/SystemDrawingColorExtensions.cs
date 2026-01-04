namespace Experiment.VulkanUtils;

using Silk.NET.Maths;
using Silk.NET.Vulkan;

public static class SystemDrawingColorExtensions
{
    public static ClearColorValue ToClearColorValue(this System.Drawing.Color color) =>
        new()
        {
            Float32_0 = color.R / 255.0f,
            Float32_1 = color.G / 255.0f,
            Float32_2 = color.B / 255.0f,
            Float32_3 = color.A / 255.0f,
        };

    public static Vector4D<byte> ToVector4Db(this System.Drawing.Color color) =>
        new(color.R, color.G, color.B, color.A);

    public static Vector4D<float> ToVector4Df(this System.Drawing.Color color) =>
        new(color.R / 255.0f, color.G / 255.0f, color.B / 255.0f, color.A / 255.0f);
}
