namespace BlazorExperiments.Lib;

public class SparseIntegerSet
{
    // TODO actually keep track of multiple ranges
    private Range? range;

    public IEnumerable<Range> Ranges
    {
        get
        {
            if (range.HasValue)
            {
                yield return range.Value;
            }
        }
    }

    public bool IsEmpty => range == null;

    public void Clear()
    {
        range = null;
    }

    public void Add(int value)
    {
        if (range == null)
        {
            range = new Range(value, value + 1);
        }
        else
        {
            range = range.Value.Expand(value);
        }
    }

    public void Add(Range range)
    {
        if (this.range == null)
        {
            this.range = range;
        }
        else
        {
            this.range = this.range.Value.Union(range);
        }
    }
}
