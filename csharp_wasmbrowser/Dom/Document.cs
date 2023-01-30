namespace Experiments.Dom;

using System.Runtime.InteropServices.JavaScript;

public partial class Document
{
	[JSImport("document.createElement", "main.js")]
	public static partial JSObject CreateElement(string tagName);
}