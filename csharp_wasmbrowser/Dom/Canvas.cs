namespace Experiments.Dom;

using System.Collections.Generic;
using System.Runtime.InteropServices.JavaScript;
using System.Text.Json;
using System.Text.Json.Nodes;

public partial class Canvas
{
	public static JSObject GetContext(JSObject canvas, string contextType)
	{
		return GetContextImpl(canvas, contextType);
	}

	public static JSObject GetContext(JSObject canvas, string contextType, WebGLContextAttributes contextAttributes)
	{
		return GetContextImpl(canvas, contextType, JsonSerializer.Serialize(contextAttributes));
	}

	[JSImport("canvas.getContext", "main.js")]
	private static partial JSObject GetContextImpl(JSObject canvas, string contextType, string? contextAttributes = null);
}
