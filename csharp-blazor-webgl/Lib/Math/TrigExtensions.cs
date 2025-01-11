using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public static class TrigExtensions<T> where T : INumber<T>, ITrigonometricFunctions<T>
{
    private static readonly Radians<T> HalfPi = new(T.Pi / T.CreateChecked(2));

    public static Radians<T> Atan2(T y, T x)
    {
        // https://en.wikipedia.org/wiki/Atan2
        return (T.Sign(x), T.Sign(y)) switch
        {
            ( > 0, _) => new(T.Atan(y / x)),
            ( < 0, >= 0) => new(T.Atan(y / x) + T.Pi),
            ( < 0, < 0) => new(T.Atan(y / x) - T.Pi),
            (0, > 0) => HalfPi,
            (0, < 0) => -HalfPi,
            (0, 0) => throw new ArgumentOutOfRangeException("atan2(0,0) is undefined"),
        };
    }
}
