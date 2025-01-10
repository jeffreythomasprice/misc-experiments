using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public record struct Degrees<T>(T Value) :
    IUnaryPlusOperators<Degrees<T>, Degrees<T>>,
    IUnaryNegationOperators<Degrees<T>, Degrees<T>>,
    IAdditionOperators<Degrees<T>, Degrees<T>, Degrees<T>>,
    ISubtractionOperators<Degrees<T>, Degrees<T>, Degrees<T>>,
    IMultiplyOperators<Degrees<T>, Degrees<T>, Degrees<T>>,
    IDivisionOperators<Degrees<T>, Degrees<T>, Degrees<T>>,
    IModulusOperators<Degrees<T>, Degrees<T>, Degrees<T>>,
    IEqualityOperators<Degrees<T>, Degrees<T>, bool>,
    IComparisonOperators<Degrees<T>, Degrees<T>, bool>
    where T : INumber<T>, ITrigonometricFunctions<T>
{
    public Radians<T> Radians => new(T.DegreesToRadians(Value));

    public static Degrees<T> operator +(Degrees<T> value)
    {
        return value;
    }

    public static Degrees<T> operator -(Degrees<T> value)
    {
        return new(-value.Value);
    }

    public static Degrees<T> operator +(Degrees<T> left, Degrees<T> right)
    {
        return new(left.Value + right.Value);
    }

    public static Degrees<T> operator -(Degrees<T> left, Degrees<T> right)
    {
        return new(left.Value - right.Value);
    }

    public static Degrees<T> operator *(Degrees<T> left, Degrees<T> right)
    {
        return new(left.Value * right.Value);
    }

    public static Degrees<T> operator /(Degrees<T> left, Degrees<T> right)
    {
        return new(left.Value / right.Value);
    }

    public static Degrees<T> operator %(Degrees<T> left, Degrees<T> right)
    {
        return new(left.Value % right.Value);
    }

    public static bool operator <(Degrees<T> left, Degrees<T> right)
    {
        return left.Value < right.Value;
    }

    public static bool operator >(Degrees<T> left, Degrees<T> right)
    {
        return left.Value > right.Value;
    }

    public static bool operator <=(Degrees<T> left, Degrees<T> right)
    {
        return left.Value <= right.Value;
    }

    public static bool operator >=(Degrees<T> left, Degrees<T> right)
    {
        return left.Value >= right.Value;
    }

    public static Degrees<T> Clamp(Degrees<T> value, Degrees<T> min, Degrees<T> max)
    {
        return new(T.Clamp(value.Value, min.Value, max.Value));
    }
}
