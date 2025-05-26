namespace Robowar.Graphics;

using Silk.NET.Maths;
using Silk.NET.OpenGL;
using StbImageSharp;

public class Texture : IDisposable
{
	private readonly GL gl;
	private readonly uint width;
	private readonly uint height;
	private readonly uint texture;

	public Texture(GL gl, Stream stream)
	{
		var image = ImageResult.FromStream(stream, ColorComponents.RedGreenBlueAlpha);

		this.gl = gl;

		width = (uint)image.Width;
		height = (uint)image.Height;

		texture = gl.GenTexture();
		gl.BindTexture(TextureTarget.Texture2D, texture);
		// TODO if not power of 2 do clamp?
		gl.TexParameter(TextureTarget.Texture2D, TextureParameterName.TextureMagFilter, (int)TextureMagFilter.Linear);
		gl.TexParameter(TextureTarget.Texture2D, TextureParameterName.TextureMinFilter, (int)TextureMagFilter.Nearest);
		gl.TexParameter(TextureTarget.Texture2D, TextureParameterName.TextureWrapS, (int)TextureWrapMode.Repeat);
		gl.TexParameter(TextureTarget.Texture2D, TextureParameterName.TextureWrapT, (int)TextureWrapMode.Repeat);
		unsafe
		{
			fixed (byte* p = &image.Data[0])
			{
				gl.TexImage2D(TextureTarget.Texture2D, 0, InternalFormat.Rgba, width, height, 0, PixelFormat.Rgba, PixelType.UnsignedByte, p);
			}
		}
	}

	public void Dispose()
	{
		gl.DeleteTexture(texture);
	}

	public Vector2D<uint> Size => new(Width, Height);

	public uint Width => width;

	public uint Height => height;

	public void Bind()
	{
		gl.BindTexture(TextureTarget.Texture2D, texture);
	}
}