namespace BlazorExperiments.WebGl;

public class Shader : IDisposable
{
    private readonly WebGL2RenderingContext gl;
    private readonly WebGL2RenderingContext.ShaderProgram program;
    private bool disposedValue;

    public Shader(WebGL2RenderingContext gl, string vertexSource, string fragmentSource)
    {
        var vertexShader = CreateShader(gl, WebGL2RenderingContext.ShaderType.VERTEX_SHADER, vertexSource);
        try
        {
            var fragmentShader = CreateShader(gl, WebGL2RenderingContext.ShaderType.FRAGMENT_SHADER, fragmentSource);
            try
            {
                var program = gl.CreateProgram();
                gl.AttachShader(program, vertexShader);
                gl.AttachShader(program, fragmentShader);
                gl.LinkProgram(program);
                gl.DetachShader(program, vertexShader);
                gl.DetachShader(program, fragmentShader);

                if (!gl.GetProgramParameter<bool>(program, WebGL2RenderingContext.LINK_STATUS))
                {
                    var log = gl.GetProgramInfoLog(program);
                    gl.DeleteProgram(program);
                    throw new Exception($"error linking shader program: {log}");
                }

                this.gl = gl;
                this.program = program;
            }
            finally
            {
                gl.DeleteShader(fragmentShader);
            }
        }
        finally
        {
            gl.DeleteShader(vertexShader);
        }
    }

    ~Shader()
    {
        // Do not change this code. Put cleanup code in 'Dispose(bool disposing)' method
        Dispose(disposing: false);
    }

    public void Dispose()
    {
        // Do not change this code. Put cleanup code in 'Dispose(bool disposing)' method
        Dispose(disposing: true);
        GC.SuppressFinalize(this);
    }

    public void UseProgram()
    {
        gl.UseProgram(program);
    }

    public int GetAttribLocation(string name)
    {
        var result = gl.GetAttribLocation(program, name);
        if (result < 0)
        {
            throw new Exception($"no such attribute: {name}");
        }
        return result;
    }

    protected virtual void Dispose(bool disposing)
    {
        if (!disposedValue)
        {
            gl.DeleteProgram(program);

            disposedValue = true;
        }
    }

    private static WebGL2RenderingContext.Shader CreateShader(WebGL2RenderingContext gl, WebGL2RenderingContext.ShaderType type, string source)
    {
        var result = gl.CreateShader(type);
        gl.ShaderSource(result, source);
        gl.CompileShader(result);

        if (!gl.GetShaderParameter<bool>(result, WebGL2RenderingContext.COMPILE_STATUS))
        {
            var log = gl.GetShaderInfoLog(result);
            gl.DeleteShader(result);
            throw new Exception($"error compiling shader: {log}");
        }

        return result;
    }
}
