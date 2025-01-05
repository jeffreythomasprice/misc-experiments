namespace BlazorExperiments.Lib;

public static class RangeExtensions
{
    public static Range Expand(this Range range, int value)
    {
        var start = System.Math.Min(range.Start.Value, value);
        var end = System.Math.Max(range.End.Value, value + 1);
        return start..end;
    }

    public static Range Union(this Range range, Range other)
    {
        var start = System.Math.Min(range.Start.Value, other.Start.Value);
        var end = System.Math.Max(range.End.Value, other.End.Value);
        return start..end;
    }
}
