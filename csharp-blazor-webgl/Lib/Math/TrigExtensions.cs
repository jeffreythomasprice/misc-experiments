using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public static class TrigExtensions
{
    public static Radians<T> Atan2<T>(T y, T x) where T : INumber<T>, ITrigonometricFunctions<T>
    {
        // https://en.wikipedia.org/wiki/Atan2
        return new((T.Sign(x), T.Sign(y)) switch
        {
            ( > 0, _) => T.Atan(y / x),
            ( < 0, >= 0) => T.Atan(y / x) + T.Pi,
            ( < 0, < 0) => T.Atan(y / x) - T.Pi,
            // TODO keep constants somewhere
            (0, > 0) => T.Pi / T.CreateChecked(2),
            (0, < 0) => -T.Pi / T.CreateChecked(2),
            (0, 0) => throw new ArgumentOutOfRangeException("atan2(0,0) is undefined"),
        });
    }
}
