namespace Robowar;

using System.Text.RegularExpressions;

public record Position(int Line, int Column)
{
	public override string ToString() => $"[{Line + 1}:{Column + 1}]";

	public Position Advance(char c) => c switch
	{
		'\n' => new Position(Line + 1, 0),
		_ when char.IsControl(c) => this,
		_ => new Position(Line, Column + 1)
	};

	public Position Advance(string s) => s.Aggregate(this, (pos, c) => pos.Advance(c));
}

public record Input(string Text, Position Position)
{
	public Input(string text) : this(text, new Position(0, 0)) { }
}

public interface Parser<T>
{
	(T result, Input remainder)? TryParse(Input input);
}

public class LiteralParser(string literal, StringComparison comparison = StringComparison.OrdinalIgnoreCase) : Parser<string>
{
	private readonly string literal = literal;
	private readonly StringComparison comparison = comparison;

	public (string result, Input remainder)? TryParse(Input input)
	{
		if (input.Text.StartsWith(literal, comparison))
		{
			var result = input.Text.Substring(0, literal.Length);
			var remainder = new Input(input.Text.Substring(literal.Length), input.Position.Advance(literal));
			return (result, remainder);
		}
		return null;
	}
}

public class RegexParser(Regex regex) : Parser<string>
{
	public (string result, Input remainder)? TryParse(Input input)
	{
		var match = regex.Match(input.Text);
		if (match.Success && match.Index == 0)
		{
			var result = match.Value;
			var remainder = new Input(input.Text.Substring(result.Length), input.Position.Advance(result));
			return (result, remainder);
		}
		return null;
	}
}

public class ChoiceParser<T>(params Parser<T>[] parsers) : Parser<T>
{
	public (T result, Input remainder)? TryParse(Input input)
	{
		foreach (var parser in parsers)
		{
			var result = parser.TryParse(input);
			if (result.HasValue)
			{
				return result;
			}
		}
		return null;
	}
}

public class SequenceParser<T>(params Parser<T>[] parsers) : Parser<T[]>
{
	public (T[] result, Input remainder)? TryParse(Input input)
	{
		var results = new List<T>();
		var currentInput = input;

		foreach (var parser in parsers)
		{
			var result = parser.TryParse(currentInput);
			if (!result.HasValue)
			{
				return null; // If any parser fails, the whole sequence fails
			}
			results.Add(result.Value.result);
			currentInput = result.Value.remainder;
		}

		return (results.ToArray(), currentInput);
	}
}