namespace Experiment.VulkanUtils;

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
}
