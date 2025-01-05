namespace BlazorExperiments.Lib.Math;

public struct ColorRGBA<T>
{
    public readonly T Red;
    public readonly T Green;
    public readonly T Blue;
    public readonly T Alpha;

    public ColorRGBA(T red, T green, T blue, T alpha)
    {
        Red = red;
        Green = green;
        Blue = blue;
        Alpha = alpha;
    }
}

public static class ColorRGBAExtensions
{
    public static ColorRGBA<byte> ToRGBA(this System.Drawing.Color color)
    {
        return new(
            color.R,
            color.G,
            color.B,
            color.A
        );
    }

    public static ColorRGBA<float> ToFloat(this ColorRGBA<byte> color)
    {
        return new(
            (float)color.Red / 255.0f,
            (float)color.Green / 255.0f,
            (float)color.Blue / 255.0f,
            (float)color.Alpha / 255.0f
        );
    }

    public static ColorRGBA<double> ToDouble(this ColorRGBA<byte> color)
    {
        return new(
            (double)color.Red / 255.0f,
            (double)color.Green / 255.0f,
            (double)color.Blue / 255.0f,
            (double)color.Alpha / 255.0f
        );
    }

    public static ColorRGBA<byte> ToByte(this ColorRGBA<float> color)
    {
        return new(
            (byte)(color.Red * 255),
            (byte)(color.Green * 255),
            (byte)(color.Blue * 255),
            (byte)(color.Alpha * 255)
        );
    }

    public static ColorRGBA<double> ToDouble(this ColorRGBA<float> color)
    {
        return new(
            (double)color.Red,
            (double)color.Green,
            (double)color.Blue,
            (double)color.Alpha
        );
    }

    public static ColorRGBA<byte> ToByte(this ColorRGBA<double> color)
    {
        return new(
            (byte)(color.Red * 255),
            (byte)(color.Green * 255),
            (byte)(color.Blue * 255),
            (byte)(color.Alpha * 255)
        );
    }

    public static ColorRGBA<float> ToFloat(this ColorRGBA<double> color)
    {
        return new(
            (float)color.Red,
            (float)color.Green,
            (float)color.Blue,
            (float)color.Alpha
        );
    }
}