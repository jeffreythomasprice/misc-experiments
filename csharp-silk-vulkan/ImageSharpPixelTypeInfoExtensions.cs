public static class ImageSharpPixelTypeInfoExtensions
{
    extension(SixLabors.ImageSharp.Formats.PixelTypeInfo pixelTypeInfo)
    {
        public int BytesPerPixel => pixelTypeInfo.BitsPerPixel / 8;
    }
}
