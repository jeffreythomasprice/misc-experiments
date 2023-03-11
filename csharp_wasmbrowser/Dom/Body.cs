namespace Experiments.Dom;

using System.Runtime.InteropServices.JavaScript;

public partial class Body
{
	public static JSObject ReplaceChildren(params JSObject[] children)
	{
		return ReplaceChildrenImpl(children);
	}

	[JSImport("body.replaceChildren", "main.js")]
	private static partial JSObject ReplaceChildrenImpl(JSObject[] children);
}
