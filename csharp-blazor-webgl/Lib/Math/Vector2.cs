using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public struct Vector2<T> :
    IAdditionOperators<Vector2<T>, Vector2<T>, Vector2<T>>,
    ISubtractionOperators<Vector2<T>, Vector2<T>, Vector2<T>>,
    IMultiplyOperators<Vector2<T>, T, Vector2<T>>,
    IDivisionOperators<Vector2<T>, T, Vector2<T>>,
    IUnaryNegationOperators<Vector2<T>, Vector2<T>>
    where T : INumber<T>
{
    public readonly T X;
    public readonly T Y;

    public Vector2(T x, T y)
    {
        X = x;
        Y = y;
    }

    public static Vector2<T> operator +(Vector2<T> left, Vector2<T> right)
    {
        return new(
            left.X + right.X,
            left.Y + right.Y
        );
    }

    public static Vector2<T> operator -(Vector2<T> left, Vector2<T> right)
    {
        return new(
            left.X - right.X,
            left.Y - right.Y
        );
    }

    public static Vector2<T> operator *(Vector2<T> left, T right)
    {
        return new(
            left.X * right,
            left.Y * right
        );
    }

    public static Vector2<T> operator /(Vector2<T> left, T right)
    {
        return new(
            left.X / right,
            left.Y / right
        );
    }

    public static Vector2<T> operator -(Vector2<T> value)
    {
        return new(-value.X, -value.Y);
    }

    public T MagnitudeSquared => X * X + Y * Y;

    public T DotProduct(Vector2<T> other)
    {
        return X * other.X + Y * other.Y;
    }
}

public static class Vector2SqrtExtensions
{
    public static T GetMagnitude<T>(this Vector2<T> v) where T : INumber<T>, IRootFunctions<T>
    {
        return T.Sqrt(v.MagnitudeSquared);
    }

    public static Vector2<T> Normalized<T>(this Vector2<T> v) where T : INumber<T>, IRootFunctions<T>
    {
        return v / v.GetMagnitude();
    }
}