using System.Text.RegularExpressions;

namespace Robowar;

public record Location(int Line, int Column)
{
	public Location Advance(char c)
	{
		if (c == '\n')
		{
			return new(Line + 1, 0);
		}
		if (Char.IsControl(c))
		{
			return this;
		}
		return new(Line, Column + 1);
	}

	public Location Advance(string s)
	{
		var result = this;
		foreach (char c in s)
		{
			result = result.Advance(c);
		}
		return result;
	}
}

public record StringInput(string String, Location Location)
{
	public (char, StringInput)? TryMatchLiteral(char c)
	{
		if (String.StartsWith(c))
		{
			var result = c;
			var remainder = new StringInput(String[1..], Location.Advance(c));
			return (result, remainder);
		}
		else
		{
			return null;
		}
	}

	public (string, StringInput)? TryMatchLiteral(string s)
	{
		if (String.StartsWith(s))
		{
			var result = s;
			var remainder = new StringInput(String[s.Length..], Location.Advance(s));
			return (result, remainder);
		}
		else
		{
			return null;
		}
	}

	public (Match, StringInput)? TryMatchRegex(Regex r)
	{
		var result = r.Match(String);
		if (result.Success && result.Index == 0)
		{
			var remainder = new StringInput(String[result.Length..], Location.Advance(result.Value));
			return (result, remainder);
		}
		else
		{
			return null;
		}
	}
}

public partial class Parser
{
	public static void Parse(StringInput input)
	{
		// TODO full parser

	}

	[GeneratedRegex("[a-zA-Z][a-zA-Z0-9]*")]
	private static partial Regex IdentifierRegex();
	private static (string, StringInput)? Identifier(StringInput input)
	{
		var match = input.TryMatchRegex(WhitespaceRegex());
		if (match == null)
		{
			return null;
		}
		else
		{
			var (result, remainder) = match.Value;
			return (result.Value, remainder);
		}
	}

	[GeneratedRegex("[ \t\n\r]+")]
	private static partial Regex WhitespaceRegex();
	private static StringInput SkipWhitespace(StringInput input)
	{
		var match = input.TryMatchRegex(WhitespaceRegex());
		if (match == null)
		{
			return input;
		}
		else
		{
			var (_, result) = match.Value;
			return result;
		}
	}
}