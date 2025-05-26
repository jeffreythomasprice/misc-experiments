namespace Robowar.Parser;

public record Location(int line, int column)
{
	public Location Advance(char c)
	{
		if (c == '\n')
		{
			return new(line + 1, 0);
		}
		if (Char.IsControl(c))
		{
			return this;
		}
		return new(line, column + 1);
	}

	public Location Advance(string s)
	{
		var result = this;
		foreach (var c in s)
		{
			result = result.Advance(c);
		}
		return result;
	}
}