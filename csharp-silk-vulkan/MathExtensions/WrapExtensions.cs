namespace Experiment.MathExtensions;

public static class WrapExtensions
{
    public static T Wrap<T>(this T value, T min, T max)
        where T : System.Numerics.INumber<T>
    {
        var range = max - min;
        var result = (value - min) % range;
        if (result < T.Zero)
        {
            result += range;
        }
        result += min;
        return result;
    }
}
