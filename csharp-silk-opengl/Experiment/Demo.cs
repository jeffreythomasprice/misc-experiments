using System.Drawing;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.OpenGL;

record struct Vertex
{
	public readonly Vector2D<float> Position;
	public readonly Vector4D<float> Color;

	public Vertex(Vector2D<float> position, Vector4D<float> color)
	{
		this.Position = position;
		this.Color = color;
	}

	public Vertex(Vector2D<float> position, Color color) : this(position, new Vector4D<float>(color.R / 255.0f, color.G / 255.0f, color.B / 255.0f, color.A / 255.0f)) { }
}

class Demo : IAppState
{
	private readonly GL gl;
	private readonly Shader shader;
	private readonly VertexArray<Vertex> vertexArray;

	public Demo(GL gl)
	{
		this.gl = gl;

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
				{ 1, new(4, VertexAttribPointerType.Float, false, nameof(Vertex.Color)) },
			}),
			[
				new(new(-0.5f, -0.5f), Color.Teal),
				new(new(-0.5f, 0.5f), Color.RebeccaPurple),
				new(new(0.5f, 0.5f), Color.Wheat),
				new(new(0.5f, -0.5f), Color.Goldenrod),
			],
			BufferUsageARB.StaticDraw,
			[
				0,1,2,
				2,3,0,
			],
			BufferUsageARB.StaticDraw
		);

		gl.ClearColor(Color.CornflowerBlue);
	}

	public void Load()
	{
	}

	public void Unload()
	{
		shader.Dispose();
		vertexArray.Dispose();
	}

	public void Resize(Vector2D<int> size)
	{
		gl.Viewport(size);

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

		vertexArray.Bind();
		shader.Use();
		unsafe
		{
			gl.DrawElements(PrimitiveType.Triangles, (uint)vertexArray.IndicesLength, DrawElementsType.UnsignedShort, null);
		}
	}
}
