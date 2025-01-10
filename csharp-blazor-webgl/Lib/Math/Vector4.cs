using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public struct Vector4<T> :
    IAdditionOperators<Vector4<T>, Vector4<T>, Vector4<T>>,
    ISubtractionOperators<Vector4<T>, Vector4<T>, Vector4<T>>,
    IMultiplyOperators<Vector4<T>, T, Vector4<T>>,
    IDivisionOperators<Vector4<T>, T, Vector4<T>>,
    IUnaryNegationOperators<Vector4<T>, Vector4<T>>
    where T : INumber<T>
{
    public readonly T X;
    public readonly T Y;
    public readonly T Z;
    public readonly T W;

    public Vector4(T x, T y, T z, T w)
    {
        X = x;
        Y = y;
        Z = z;
        W = w;
    }

    public override string ToString()
    {
        return $"({X}, {Y}, {Z}, {W})";
    }

    public static Vector4<T> operator +(Vector4<T> left, Vector4<T> right)
    {
        return new(
            left.X + right.X,
            left.Y + right.Y,
            left.Z + right.Z,
            left.W + right.W
        );
    }

    public static Vector4<T> operator -(Vector4<T> left, Vector4<T> right)
    {
        return new(
            left.X - right.X,
            left.Y - right.Y,
            left.Z - right.Z,
            left.W - right.W
        );
    }

    public static Vector4<T> operator *(Vector4<T> left, T right)
    {
        return new(
            left.X * right,
            left.Y * right,
            left.Z * right,
            left.W * right
        );
    }

    public static Vector4<T> operator /(Vector4<T> left, T right)
    {
        return new(
            left.X / right,
            left.Y / right,
            left.Z / right,
            left.W / right
        );
    }

    public static Vector4<T> operator -(Vector4<T> value)
    {
        return new(-value.X, -value.Y, -value.Z, -value.W);
    }

    public T MagnitudeSquared => X * X + Y * Y + Z * Z + W * W;

    public T DotProduct(Vector4<T> other)
    {
        return X * other.X + Y * other.Y + Z * other.Z + W * other.W;
    }
}

public static class Vector4Extensions
{
    public static T GetMagnitude<T>(this Vector4<T> v) where T : INumber<T>, IRootFunctions<T>
    {
        return T.Sqrt(v.MagnitudeSquared);
    }

    public static Vector4<T> Normalized<T>(this Vector4<T> v) where T : INumber<T>, IRootFunctions<T>
    {
        return v / v.GetMagnitude();
    }
}