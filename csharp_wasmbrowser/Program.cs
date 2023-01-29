using System;
using System.Runtime.InteropServices.JavaScript;

Console.WriteLine("Hello, Browser!");

public partial class Foobar
{
	[JSExport]
	internal static void Main()
	{
		Console.WriteLine("TODO JEFF here");
		// TODO leaking, should dispose?
		var div = CreateElement("div");
		Console.WriteLine($"TODO JEFF div = {div}");
	}

	[JSImport("document.createElement", "main.js")]
	internal static partial JSObject CreateElement(string tagName);
}
