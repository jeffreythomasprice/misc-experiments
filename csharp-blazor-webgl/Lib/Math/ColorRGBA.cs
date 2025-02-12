﻿using System.Diagnostics.CodeAnalysis;
using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public struct ColorRGBA<T> :
    IEqualityOperators<ColorRGBA<T>, ColorRGBA<T>, bool>
    where T :
        IEqualityOperators<T, T, bool>
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

    public override string ToString()
    {
        return $"RGBA({Red}, {Green}, {Blue}, {Alpha})";
    }

    public override bool Equals([NotNullWhen(true)] object? obj)
    {
        if (obj is ColorRGBA<T> v)
        {
            return Equals(v);
        }
        else
        {
            return false;
        }
    }

    public bool Equals(ColorRGBA<T> other)
    {
        return this == other;
    }

    public static bool operator ==(ColorRGBA<T> left, ColorRGBA<T> right)
    {
        return left.Red == right.Red && left.Green == right.Green && left.Blue == right.Blue && left.Alpha == right.Alpha;
    }

    public static bool operator !=(ColorRGBA<T> left, ColorRGBA<T> right)
    {
        return !(left == right);
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