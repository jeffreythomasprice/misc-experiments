using System;
using System.Collections.Generic;
using System.Drawing;
using System.Reflection;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.OpenGL;

class Demo : IAppState
{
	private readonly GL gl;
	private readonly Shader shader;
	private readonly VertexArray<Vector2D<float>> vertexArray;

	public Demo(GL gl)
	{
		this.gl = gl;

		shader = new Shader(
			gl,
			EmbeddedFileAsString("Experiment.Assets.Shaders.shader.vert"),
			EmbeddedFileAsString("Experiment.Assets.Shaders.shader.frag")
		);

		vertexArray = new VertexArray<Vector2D<float>>(
			gl,
			new([
				new VertexAttributeSpecification<Vector2D<float>>(0, 2, VertexAttribPointerType.Float, false, 0),
			]),
			[
				new(-0.5f, -0.5f),
				new(0.5f,-0.5f),
				new(0.0f,0.5f),
			],
			BufferUsageARB.StaticDraw,
			[
				0,1,2,
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
		// TODO animate stuff
		return null;
	}

	public void Render()
	{
		gl.Clear(ClearBufferMask.ColorBufferBit);

		vertexArray.Bind();
		shader.Use();
		unsafe
		{
			gl.DrawElements(PrimitiveType.Triangles, 3, DrawElementsType.UnsignedShort, null);
		}
	}

	// TODO move me?
	private static string EmbeddedFileAsString(string name)
	{
		using var stream = Assembly.GetExecutingAssembly().GetManifestResourceStream(name);
		if (stream == null)
		{
			throw new Exception($"failed to find embedded file: {name}");
		}
		using var reader = new StreamReader(stream);
		return reader.ReadToEnd();
	}
}
