using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public struct Vector2<T> :
    IAdditionOperators<Vector2<T>, Vector2<T>, Vector2<T>>,
    ISubtractionOperators<Vector2<T>, Vector2<T>, Vector2<T>>,
    IMultiplyOperators<Vector2<T>, T, Vector2<T>>,
    IDivisionOperators<Vector2<T>, T, Vector2<T>>
    where T : INumber<T>
{
    public required T X { get; init; }
    public required T Y { get; init; }

    public static Vector2<T> operator +(Vector2<T> left, Vector2<T> right)
    {
        return new()
        {
            X = left.X + right.X,
            Y = left.Y + right.Y
        };
    }

    public static Vector2<T> operator -(Vector2<T> left, Vector2<T> right)
    {
        return new()
        {
            X = left.X - right.X,
            Y = left.Y - right.Y
        };
    }

    public static Vector2<T> operator *(Vector2<T> left, T right)
    {
        return new()
        {
            X = left.X * right,
            Y = left.Y * right
        };
    }

    public static Vector2<T> operator /(Vector2<T> left, T right)
    {
        return new()
        {
            X = left.X / right,
            Y = left.Y / right
        };
    }

    // TODO magnitude
    // TODO dot product
}
