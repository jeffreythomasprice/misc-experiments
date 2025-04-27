using System.Drawing;
using System.Numerics;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.OpenGL;

record struct Vertex
{
	public readonly Vector2D<float> Position;
	public readonly Vector2D<float> TextureCoordinate;
	public readonly Vector4D<float> Color;

	public Vertex(Vector2D<float> position, Vector2D<float> textureCoordinate, Vector4D<float> color)
	{
		this.Position = position;
		this.TextureCoordinate = textureCoordinate;
		this.Color = color;
	}

	public Vertex(Vector2D<float> position, Vector2D<float> textureCoordinate, Color color)
		: this(position, textureCoordinate, new Vector4D<float>(color.R / 255.0f, color.G / 255.0f, color.B / 255.0f, color.A / 255.0f)) { }
}

class Demo : IAppState
{
	private readonly GL gl;
	private readonly IWindowState windowState;

	private readonly Texture texture;
	private readonly Shader shader;
	private readonly VertexArray<Vertex> vertexArray;

	private Matrix4X4<float> orthoMatrix;

	public Demo(GL gl, IWindowState windowState)
	{
		this.gl = gl;
		this.windowState = windowState;

		using var image = App.EmbeddedFileAsStream("Experiment.Assets.silknet.png");
		texture = new Texture(gl, image);

		shader = new Shader(
			gl,
			App.EmbeddedFileAsString("Experiment.Assets.Shaders.shader.vert"),
			App.EmbeddedFileAsString("Experiment.Assets.Shaders.shader.frag")
		);

		vertexArray = new(
			gl,
			new(new Dictionary<uint, VertexAttributeSpecification<Vertex>>()
			{
				{ 0, new(2, VertexAttribPointerType.Float, false, nameof(Vertex.Position)) },
				{ 1, new(2, VertexAttribPointerType.Float, false, nameof(Vertex.TextureCoordinate)) },
				{ 2, new(4, VertexAttribPointerType.Float, false, nameof(Vertex.Color)) },
			}),
			[
				new(
					new(0.0f, 0.0f),
					new(0.0f, 0.0f),
					Color.Teal
				),
				new(
					new((float)texture.Width, 0.0f),
					new(1.0f, 0.0f),
					Color.RebeccaPurple
				),
				new(
					new((float)texture.Width, (float)texture.Height),
					new(1.0f, 1.0f),
					Color.Wheat
				),
				new(
					new(0.0f, (float)texture.Height),
					new(0.0f, 1.0f),
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
		shader.Dispose();
		vertexArray.Dispose();
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

		shader.Use();

		gl.UniformMatrix4(shader.GetUniformLocation("projectionMatrixUniform"), false, orthoMatrix.ToArray());

		gl.ActiveTexture(TextureUnit.Texture0);
		texture.Bind();
		gl.Uniform1(shader.GetUniformLocation("samplerUniform"), 0);

		vertexArray.Bind();
		unsafe
		{
			gl.DrawElements(PrimitiveType.Triangles, (uint)vertexArray.IndicesLength, DrawElementsType.UnsignedShort, null);
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