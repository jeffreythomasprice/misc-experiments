using System.Reflection;

static class AssemblyExtensions
{
	public static string SanitizeManifestResourceName(this Assembly assembly, string name)
	{
		string SanitizeName(string name)
		{
			return name
				.Replace('/', '*')
				.Replace('-', '*')
				.Replace('_', '*')
				.Replace('.', '*')
				.ToLower();
		}

		var sanitizedName = SanitizeName(name);
		var results = assembly.GetManifestResourceNames().Where(resourceName =>
		{
			return SanitizeName(resourceName) == sanitizedName;
		}).ToList();
		if (results.Count == 1)
		{
			return results[0];
		}
		if (results.Count == 0)
		{
			throw new Exception($"can't find name for: {name}");
		}
		throw new Exception($"multiple results for: {name}");
	}

	public static Stream AssertManifestResourceStream(this Assembly assembly, string name)
	{
		return assembly.GetManifestResourceStream(assembly.SanitizeManifestResourceName(name))
			?? throw new Exception($"failed to find embedded file: {name}");
	}

	public static string AssertManifestResourceString(this Assembly assembly, string name)
	{
		using var stream = AssertManifestResourceStream(assembly, name);
		using var reader = new StreamReader(stream);
		return reader.ReadToEnd();
	}
}
