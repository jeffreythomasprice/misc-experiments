using Silk.NET.Maths;

public static class ColorExtensions
{
	public static Silk.NET.WebGPU.Color ToWebGPU(this System.Drawing.Color c)
	{
		return new(
			c.R / 255.0,
			c.G / 255.0,
			c.B / 255.0,
			c.A / 255.0
		);
	}

	public static Vector4D<float> ToVector(this System.Drawing.Color c)
	{
		return new(
			c.R / 255.0f,
			c.G / 255.0f,
			c.B / 255.0f,
			c.A / 255.0f
		);
	}
}