namespace Robowar.Graphics;

using Silk.NET.OpenGL;

public class Shader : IDisposable
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

	public int GetUniformLocation(string name)
	{
		var result = gl.GetUniformLocation(program, name);
		if (result < 0)
		{
			throw new Exception($"no such uniform: {name}");
		}
		return result;
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