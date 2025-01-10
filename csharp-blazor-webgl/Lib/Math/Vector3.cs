using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public struct Vector3<T> :
    IAdditionOperators<Vector3<T>, Vector3<T>, Vector3<T>>,
    ISubtractionOperators<Vector3<T>, Vector3<T>, Vector3<T>>,
    IMultiplyOperators<Vector3<T>, T, Vector3<T>>,
    IDivisionOperators<Vector3<T>, T, Vector3<T>>,
    IUnaryNegationOperators<Vector3<T>, Vector3<T>>
    where T : INumber<T>
{
    public readonly T X;
    public readonly T Y;
    public readonly T Z;

    public Vector3(T x, T y, T z)
    {
        X = x;
        Y = y;
        Z = z;
    }

    public override string ToString()
    {
        return $"({X}, {Y}, {Z})";
    }

    public static Vector3<T> operator +(Vector3<T> left, Vector3<T> right)
    {
        return new(
            left.X + right.X,
            left.Y + right.Y,
            left.Z + right.Z
        );
    }

    public static Vector3<T> operator -(Vector3<T> left, Vector3<T> right)
    {
        return new(
            left.X - right.X,
            left.Y - right.Y,
            left.Z - right.Z
        );
    }

    public static Vector3<T> operator *(Vector3<T> left, T right)
    {
        return new(
            left.X * right,
            left.Y * right,
            left.Z * right
        );
    }

    public static Vector3<T> operator /(Vector3<T> left, T right)
    {
        return new(
            left.X / right,
            left.Y / right,
            left.Z / right
        );
    }

    public static Vector3<T> operator -(Vector3<T> value)
    {
        return new(-value.X, -value.Y, -value.Z);
    }

    public T MagnitudeSquared => X * X + Y * Y + Z * Z;

    public T DotProduct(Vector3<T> other)
    {
        return X * other.X + Y * other.Y + Z * other.Z;
    }

    public Vector3<T> CrossProduct(Vector3<T> other)
    {
        return new(
            Y * other.Z - Z * other.Y,
            Z * other.X - X * other.Z,
            X * other.Y - Y * other.X
        );
    }
}

public static class Vector3Extensions
{
    public static T GetMagnitude<T>(this Vector3<T> v) where T : INumber<T>, IRootFunctions<T>
    {
        return T.Sqrt(v.MagnitudeSquared);
    }

    public static Vector3<T> Normalized<T>(this Vector3<T> v) where T : INumber<T>, IRootFunctions<T>
    {
        return v / v.GetMagnitude();
    }

    public static Radians<T> AngleBetween<T>(this Vector3<T> v, Vector3<T> other) where T : INumber<T>, IRootFunctions<T>, ITrigonometricFunctions<T>
    {
        /*
        https://stackoverflow.com/a/16544330/9290998
        https://stackoverflow.com/a/67719217/9290998
        x = dot(a, b)
        y = dot(n, cross(a, b))
        angle = atan2(y, x)
        */
        return TrigExtensions.Atan2(
            v.CrossProduct(other).GetMagnitude(),
            v.DotProduct(other)
        );
    }
}