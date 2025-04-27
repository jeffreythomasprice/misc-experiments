using System.Drawing;
using System.Reflection;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.OpenGL;

class Demo : IAppState
{
	private readonly GL gl;
	private readonly Shader shader;
	private readonly uint vertexArray;

	public Demo(GL gl)
	{
		this.gl = gl;

		shader = new Shader(
			gl,
			EmbeddedFileAsString("Experiment.Assets.Shaders.shader.vert"),
			EmbeddedFileAsString("Experiment.Assets.Shaders.shader.frag")
		);

		vertexArray = gl.GenVertexArray();
		gl.BindVertexArray(vertexArray);
		var arrayBuffer = gl.GenBuffer();
		gl.BindBuffer(BufferTargetARB.ArrayBuffer, arrayBuffer);
		var vertices = new float[] {
			-0.5f, -0.5f,
			0.5f, -0.5f,
			0.0f, 0.5f,
		};
		gl.BufferData<float>(BufferTargetARB.ArrayBuffer, vertices, BufferUsageARB.StaticDraw);
		var elementArrayBuffer = gl.GenBuffer();
		gl.BindBuffer(BufferTargetARB.ElementArrayBuffer, elementArrayBuffer);
		var indices = new UInt16[] {
			0, 1, 2
		};
		gl.BufferData<UInt16>(BufferTargetARB.ElementArrayBuffer, indices, BufferUsageARB.StaticDraw);

		gl.VertexAttribPointer(0, 2, VertexAttribPointerType.Float, false, 2 * sizeof(float), 0);
		gl.EnableVertexAttribArray(0);

		gl.ClearColor(Color.CornflowerBlue);
	}

	public void Load()
	{
	}

	public void Unload()
	{
		shader.Dispose();
		// TODO buffers
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

		gl.BindVertexArray(vertexArray);
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

// TODO move me
class Shader : IDisposable
{
	private readonly GL gl;
	private readonly uint vertexShader;
	private readonly uint fragmentShader;
	private readonly uint program;

	public Shader(GL gl, string vertexSource, string fragmentSource)
	{
		this.gl = gl;

		vertexShader = CreateShader(gl, ShaderType.VertexShader, vertexSource);
		try
		{
			fragmentShader = CreateShader(gl, ShaderType.FragmentShader, fragmentSource);
			try
			{
				program = gl.CreateProgram();
				gl.AttachShader(program, vertexShader);
				gl.AttachShader(program, fragmentShader);
				gl.LinkProgram(program);
				if (gl.GetProgram(program, ProgramPropertyARB.LinkStatus) == 0)
				{
					var log = gl.GetProgramInfoLog(program);
					gl.DeleteProgram(program);
					throw new Exception($"error linking shader program: {log}");
				}
			}
			catch
			{
				gl.DeleteShader(fragmentShader);
			}
		}
		catch
		{
			gl.DeleteShader(vertexShader);
			throw;
		}
	}

	public void Dispose()
	{
		gl.DeleteShader(vertexShader);
		gl.DeleteShader(fragmentShader);
		gl.DeleteProgram(program);
	}

	public void Use()
	{
		gl.UseProgram(program);
	}

	private static uint CreateShader(GL gl, ShaderType type, string source)
	{
		var result = gl.CreateShader(type);
		gl.ShaderSource(result, source);
		gl.CompileShader(result);
		if (gl.GetShader(result, ShaderParameterName.CompileStatus) == 0)
		{
			var log = gl.GetShaderInfoLog(result);
			gl.DeleteShader(result);
			throw new Exception($"compile error for shader of type {type}: {log}");
		}
		return result;
	}
}