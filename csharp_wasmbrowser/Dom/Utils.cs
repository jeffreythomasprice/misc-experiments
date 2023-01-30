namespace Experiments.Dom;

using System.Runtime.InteropServices.JavaScript;

public partial class Utils
{
	[JSImport("utils.createObject", "main.js")]
	public static partial JSObject CreateObject();
}