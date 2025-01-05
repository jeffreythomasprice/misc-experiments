namespace BlazorExperiments.Lib.Math;

public struct ColorRGBA<T>
{
    public required T Red { get; init; }
    public required T Green { get; init; }
    public required T Blue { get; init; }
    public required T Alpha { get; init; }

    public ColorRGBA() { }
}

public static class ColorRGBAExtensions
{
    public static ColorRGBA<byte> ToRGBA(this System.Drawing.Color color)
    {
        return new()
        {
            Red = color.R,
            Green = color.G,
            Blue = color.B,
            Alpha = color.A
        };
    }

    public static ColorRGBA<float> ToFloat(this ColorRGBA<byte> color)
    {
        return new()
        {
            Red = (float)color.Red / 255.0f,
            Green = (float)color.Green / 255.0f,
            Blue = (float)color.Blue / 255.0f,
            Alpha = (float)color.Alpha / 255.0f
        };
    }

    public static ColorRGBA<double> ToDouble(this ColorRGBA<byte> color)
    {
        return new()
        {
            Red = (double)color.Red / 255.0f,
            Green = (double)color.Green / 255.0f,
            Blue = (double)color.Blue / 255.0f,
            Alpha = (double)color.Alpha / 255.0f
        };
    }

    public static ColorRGBA<byte> ToByte(this ColorRGBA<float> color)
    {
        return new()
        {
            Red = (byte)(color.Red * 255),
            Green = (byte)(color.Green * 255),
            Blue = (byte)(color.Blue * 255),
            Alpha = (byte)(color.Alpha * 255)
        };
    }

    public static ColorRGBA<double> ToDouble(this ColorRGBA<float> color)
    {
        return new()
        {
            Red = (double)color.Red,
            Green = (double)color.Green,
            Blue = (double)color.Blue,
            Alpha = (double)color.Alpha
        };
    }

    public static ColorRGBA<byte> ToByte(this ColorRGBA<double> color)
    {
        return new()
        {
            Red = (byte)(color.Red * 255),
            Green = (byte)(color.Green * 255),
            Blue = (byte)(color.Blue * 255),
            Alpha = (byte)(color.Alpha * 255)
        };
    }

    public static ColorRGBA<float> ToFloat(this ColorRGBA<double> color)
    {
        return new()
        {
            Red = (float)color.Red,
            Green = (float)color.Green,
            Blue = (float)color.Blue,
            Alpha = (float)color.Alpha
        };
    }
}