using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public record struct Radians<T>(T Value) :
    IUnaryPlusOperators<Radians<T>, Radians<T>>,
    IUnaryNegationOperators<Radians<T>, Radians<T>>,
    IAdditionOperators<Radians<T>, Radians<T>, Radians<T>>,
    ISubtractionOperators<Radians<T>, Radians<T>, Radians<T>>,
    IMultiplyOperators<Radians<T>, Radians<T>, Radians<T>>,
    IDivisionOperators<Radians<T>, Radians<T>, Radians<T>>,
    IModulusOperators<Radians<T>, Radians<T>, Radians<T>>,
    IEqualityOperators<Radians<T>, Radians<T>, bool>,
    IComparisonOperators<Radians<T>, Radians<T>, bool>
    where T : INumber<T>, ITrigonometricFunctions<T>
{
    public Degrees<T> Degrees => new(T.RadiansToDegrees(Value));

    public static Radians<T> operator +(Radians<T> value)
    {
        return value;
    }

    public static Radians<T> operator -(Radians<T> value)
    {
        return new(-value.Value);
    }

    public static Radians<T> operator +(Radians<T> left, Radians<T> right)
    {
        return new(left.Value + right.Value);
    }

    public static Radians<T> operator -(Radians<T> left, Radians<T> right)
    {
        return new(left.Value - right.Value);
    }

    public static Radians<T> operator *(Radians<T> left, Radians<T> right)
    {
        return new(left.Value * right.Value);
    }

    public static Radians<T> operator /(Radians<T> left, Radians<T> right)
    {
        return new(left.Value / right.Value);
    }

    public static Radians<T> operator %(Radians<T> left, Radians<T> right)
    {
        return new(left.Value % right.Value);
    }

    public static bool operator <(Radians<T> left, Radians<T> right)
    {
        return left.Value < right.Value;
    }

    public static bool operator >(Radians<T> left, Radians<T> right)
    {
        return left.Value > right.Value;
    }

    public static bool operator <=(Radians<T> left, Radians<T> right)
    {
        return left.Value <= right.Value;
    }

    public static bool operator >=(Radians<T> left, Radians<T> right)
    {
        return left.Value >= right.Value;
    }

    public static Radians<T> Clamp(Radians<T> value, Radians<T> min, Radians<T> max)
    {
        return new(T.Clamp(value.Value, min.Value, max.Value));
    }
}
