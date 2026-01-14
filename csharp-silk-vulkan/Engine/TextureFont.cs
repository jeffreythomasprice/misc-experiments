namespace Experiment.Engine;

using Experiment.VulkanUtils;
using Microsoft.Extensions.Logging;
using Silk.NET.Vulkan;
using SixLabors.Fonts;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.Drawing.Processing;
using SixLabors.ImageSharp.PixelFormats;
using SixLabors.ImageSharp.Processing;

public sealed class TextureFont
{
    private readonly Font font;

    public TextureFont(string path, float size)
    {
        var fontCollection = new FontCollection();
        var family = fontCollection.Add(path);
        font = family.CreateFont(size);
    }

    // TODO a version of this that can re-render to an existing image and return how big the sub-image on that texture is that holds real data
    public TextureImageWrapper DrawString(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        string s
    )
    {
        var bounds = TextMeasurer.MeasureAdvance(s, new(font));
        var image = new Image<Rgba32>(
            (int)Math.Ceiling(bounds.Width),
            (int)Math.Ceiling(bounds.Height)
        );
        image.Mutate(context =>
        {
            context.Clear(Color.Transparent);
            context.DrawText(s, font, Color.White, new(0, 0));
        });
        return new TextureImageWrapper(vk, physicalDevice, device, commandPool, image);
    }
}
