using System.Drawing;
using System.Numerics;
using System.Reflection;
using Robowar.Graphics;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.OpenGL;

record struct VertexPosition2DTexturedColored
{
	public readonly Vector2D<float> Position;
	public readonly Vector2D<float> TextureCoordinate;
	public readonly Vector4D<float> Color;

	public VertexPosition2DTexturedColored(Vector2D<float> position, Vector2D<float> textureCoordinate, Vector4D<float> color)
	{
		this.Position = position;
		this.TextureCoordinate = textureCoordinate;
		this.Color = color;
	}

	public VertexPosition2DTexturedColored(Vector2D<float> position, Vector2D<float> textureCoordinate, Color color)
		: this(position, textureCoordinate, new Vector4D<float>(color.R / 255.0f, color.G / 255.0f, color.B / 255.0f, color.A / 255.0f)) { }
}

record struct VertexPosition2DColored
{
	public readonly Vector2D<float> Position;
	public readonly Vector4D<float> Color;

	public VertexPosition2DColored(Vector2D<float> position, Vector4D<float> color)
	{
		this.Position = position;
		this.Color = color;
	}

	public VertexPosition2DColored(Vector2D<float> position, Color color)
		: this(position, new Vector4D<float>(color.R / 255.0f, color.G / 255.0f, color.B / 255.0f, color.A / 255.0f)) { }
}

class Demo : IAppState
{
	private readonly GL gl;
	private readonly IWindowState windowState;

	private readonly Robowar.Graphics.Texture texture;
	private readonly Robowar.Graphics.Shader shaderPosition2DTexturedColored;
	private readonly Robowar.Graphics.Shader shaderPosition2DColored;
	private readonly VertexArray<VertexPosition2DTexturedColored> vertexArrayTexturedColored;
	private readonly VertexArray<VertexPosition2DColored> vertexArrayColored;

	private Matrix4X4<float> orthoMatrix;

	public Demo(GL gl, IWindowState windowState)
	{
		this.gl = gl;
		this.windowState = windowState;

		using var image = Assembly.GetExecutingAssembly().AssertManifestResourceStream("Robowar.Assets.silknet.png");
		texture = new Robowar.Graphics.Texture(gl, image);

		shaderPosition2DTexturedColored = new Robowar.Graphics.Shader(
			gl,
			Assembly.GetExecutingAssembly().AssertManifestResourceString("Robowar.Assets.Shaders.position2DTexturedColored.vert"),
			Assembly.GetExecutingAssembly().AssertManifestResourceString("Robowar.Assets.Shaders.position2DTexturedColored.frag")
		);
		shaderPosition2DColored = new Robowar.Graphics.Shader(
			gl,
			Assembly.GetExecutingAssembly().AssertManifestResourceString("Robowar.Assets.Shaders.position2DColored.vert"),
			Assembly.GetExecutingAssembly().AssertManifestResourceString("Robowar.Assets.Shaders.position2DColored.frag")
		);

		vertexArrayTexturedColored = new(
			gl,
			new(new Dictionary<uint, VertexAttributeSpecification<VertexPosition2DTexturedColored>>()
			{
				{ 0, new(2, VertexAttribPointerType.Float, false, nameof(VertexPosition2DTexturedColored.Position)) },
				{ 1, new(2, VertexAttribPointerType.Float, false, nameof(VertexPosition2DTexturedColored.TextureCoordinate)) },
				{ 2, new(4, VertexAttribPointerType.Float, false, nameof(VertexPosition2DTexturedColored.Color)) },
			}),
			[
				new(
					new(0.0f, 0.0f),
					new(0.0f, 0.0f),
					Color.Red
				),
				new(
					new((float)texture.Width, 0.0f),
					new(1.0f, 0.0f),
					Color.Blue
				),
				new(
					new((float)texture.Width, (float)texture.Height),
					new(1.0f, 1.0f),
					Color.Blue
				),
				new(
					new(0.0f, (float)texture.Height),
					new(0.0f, 1.0f),
					Color.Red
				),
			],
			BufferUsageARB.StaticDraw,
			[
				0,1,2,
				2,3,0,
			],
			BufferUsageARB.StaticDraw
		);
		vertexArrayColored = new(
			gl,
			new(new Dictionary<uint, VertexAttributeSpecification<VertexPosition2DColored>>()
			{
				{ 0, new(2, VertexAttribPointerType.Float, false, nameof(VertexPosition2DColored.Position)) },
				{ 1, new(4, VertexAttribPointerType.Float, false, nameof(VertexPosition2DColored.Color)) },
			}),
			[
				new(
					new(100,100),
					Color.Teal
				),
				new(
					new(300,100),
					Color.RebeccaPurple
				),
				new(
					new(300,300),
					Color.Wheat
				),
				new(
					new(100,300),
					Color.Goldenrod
				),
			],
			BufferUsageARB.StaticDraw,
			[
				0,1,2,
				2,3,0,
			],
			BufferUsageARB.StaticDraw
		);

		gl.ClearColor(Color.CornflowerBlue);

		orthoMatrix = Matrix4X4.CreateOrthographicOffCenter(0.0f, (float)windowState.Size.X, (float)windowState.Size.Y, 0.0f, -1.0f, 1.0f);
	}

	public void Load()
	{
	}

	public void Unload()
	{
		texture.Dispose();
		shaderPosition2DTexturedColored.Dispose();
		shaderPosition2DColored.Dispose();
		vertexArrayTexturedColored.Dispose();
		vertexArrayColored.Dispose();
	}

	public void Resize(Vector2D<int> size)
	{
		gl.Viewport(size);

		// TODO deduplicate with the constructor
		orthoMatrix = Matrix4X4.CreateOrthographicOffCenter(0.0f, (float)size.X, (float)size.Y, 0.0f, -1.0f, 1.0f);
	}

	public AppStateTransition? KeyDown(Key key)
	{
		return null;
	}

	public AppStateTransition? KeyUp(Key key)
	{
		if (key == Key.Escape)
		{
			return AppStateTransition.Exit;
		}
		return null;
	}

	public AppStateTransition? Update(TimeSpan delta)
	{
		return null;
	}

	public void Render()
	{
		gl.Clear(ClearBufferMask.ColorBufferBit);

		shaderPosition2DTexturedColored.Use();

		gl.UniformMatrix4(shaderPosition2DTexturedColored.GetUniformLocation("projectionMatrixUniform"), false, orthoMatrix.ToArray());

		gl.ActiveTexture(TextureUnit.Texture0);
		texture.Bind();
		gl.Uniform1(shaderPosition2DTexturedColored.GetUniformLocation("samplerUniform"), 0);

		vertexArrayTexturedColored.Bind();
		unsafe
		{
			gl.DrawElements(PrimitiveType.Triangles, (uint)vertexArrayTexturedColored.IndicesLength, DrawElementsType.UnsignedShort, null);
		}

		shaderPosition2DColored.Use();

		gl.UniformMatrix4(shaderPosition2DColored.GetUniformLocation("projectionMatrixUniform"), false, orthoMatrix.ToArray());

		vertexArrayColored.Bind();
		unsafe
		{
			gl.DrawElements(PrimitiveType.Triangles, (uint)vertexArrayColored.IndicesLength, DrawElementsType.UnsignedShort, null);
		}
	}
}

static class Matrix4X4Extensions
{
	public static T[] ToArray<T>(this Matrix4X4<T> m) where T : unmanaged, IFormattable, IEquatable<T>, IComparable<T>
	{
		return [
			m.M11, m.M12, m.M13, m.M14,
			m.M21, m.M22, m.M23, m.M24,
			m.M31, m.M32, m.M33, m.M34,
			m.M41, m.M42, m.M43, m.M44,
		];
	}
}