namespace Experiment;

public static class Env
{
	public static string AssertString(string name)
	{
		var result = Environment.GetEnvironmentVariable(name);
		if (string.IsNullOrWhiteSpace(result))
		{
			throw new Exception($"missing environment variable: {name}");
		}
		return result;
	}

	public static int AssertInt(string name)
	{
		var s = AssertString(name);
		if (!int.TryParse(s, out var result))
		{
			throw new Exception($"got environment variable {name}, but isn't an integer");
		}
		return result;
	}
}
