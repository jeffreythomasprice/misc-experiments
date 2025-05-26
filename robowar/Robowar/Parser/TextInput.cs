namespace Robowar.Parser;

public class TextInput
{
	public interface IMatcher<T>
	{
		public T Match();
	}

	private record FuncMatcher<T>(Func<T> Func) : IMatcher<T>
	{
		public T Match() => Func();
	}

	public class ParseException : Exception
	{
		public readonly Location Location;

		public ParseException(Location location, string message) : base(message)
		{
			Location = location;
		}
	}

	public interface ISavedLocation
	{
		void Restore();
	}

	private class SavedLocation : ISavedLocation
	{
		public required TextInput parent { get; init; }
		public required string remainingInput { get; init; }
		public required Location location { get; init; }

		public void Restore()
		{
			parent.remainingInput = remainingInput;
			parent.location = location;
		}
	}

	private string remainingInput;
	private Location location;

	public TextInput(string input)
	{
		remainingInput = input;
		location = new(0, 0);
	}

	public ISavedLocation GetSavedPoint()
	{
		return new SavedLocation()
		{
			parent = this,
			remainingInput = remainingInput,
			location = location,
		};
	}

	public T SafeParse<T>(Func<T> f)
	{
		var saved = GetSavedPoint();
		try
		{
			return f();
		}
		catch
		{
			saved.Restore();
			throw;
		}
	}

	// TODO Literal that takes char

	public IMatcher<string> Literal(string expected)
	{
		return new FuncMatcher<string>(() =>
		{
			if (!remainingInput.StartsWith(expected))
			{
				throw new ParseException(location, $"expected \"{expected}\"");
			}
			var matched = remainingInput.Substring(0, expected.Length);
			remainingInput = remainingInput.Substring(expected.Length);
			location = location.Advance(matched);
			return matched;
		});
	}

	// TODO Literal that takes StringComparison comparisonType	var results = Sequence(() => this.Literal("foo"), () => this.Literal("bar"));

	public IMatcher<(T1, T2)> Sequence<T1, T2>(IMatcher<T1> m1, IMatcher<T2> m2)
	{
		return new FuncMatcher<(T1, T2)>(() =>
		{
			return SafeParse(() =>
			{
				var r1 = m1.Match();
				var r2 = m2.Match();
				return (r1, r2);
			});
		});
	}

	// TODO delete me
	private void test()
	{
		var matcher = Sequence(Literal("foo"), Literal("bar"));
	}

	/*
			TODO how to do arbitrary text input?

			be able to save and restore a location
			get a save point
			from a save point get what location this was at
			restore the save point

			match some string and update the text point
			*/
}
