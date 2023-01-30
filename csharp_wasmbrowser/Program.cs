using System;
using System.Runtime.InteropServices.JavaScript;
using Experiments.Dom;

var canvas = Document.CreateElement("canvas");
var canvasStyle = canvas.GetPropertyAsJSObject("style");
canvasStyle!.SetProperty("position", "absolute");
canvasStyle.SetProperty("width", "100%");
canvasStyle.SetProperty("height", "100%");
canvasStyle.SetProperty("left", "0");
canvasStyle.SetProperty("top", "0");
Body.ReplaceChildren(new[] { canvas });

var context = Canvas.GetContext(canvas, "webgl2", new WebGLContextAttributes
{
	PowerPreference = WebGLContextAttributes.PowerPreferenceType.HighPerformance,
});
