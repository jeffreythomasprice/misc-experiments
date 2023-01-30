namespace Experiments.Dom;

using System.Runtime.InteropServices.JavaScript;

public partial class Utils
{
	public delegate void AnimationDelegate(double time);

	public static event AnimationDelegate? OnAnimate;

	[JSImport("utils.startAnimation", "main.js")]
	public static partial void StartAnimation();

	[JSImport("utils.stopAnimation", "main.js")]
	public static partial void StopAnimation();

	[JSExport]
	private static void Animate(double time)
	{
		OnAnimate?.Invoke(time);
	}
}