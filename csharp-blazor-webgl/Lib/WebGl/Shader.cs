namespace BlazorExperiments.Lib.WebGl;

public class Shader : IDisposable
{
    public record class Attribute(WebGL2RenderingContext.ActiveInfo Info, int Location);

    public record class Uniform(Shader shader, WebGL2RenderingContext.ActiveInfo Info, WebGL2RenderingContext.UniformLocation Location)
    {
        public void Set(int value)
        {
            shader.gl.Uniform1i(shader.program, Location, value);
        }

        // TODO more uniforms uniform[1234][fi][v]()
    }

    private readonly WebGL2RenderingContext gl;
    private readonly WebGL2RenderingContext.ShaderProgram program;
    private bool disposedValue;

    public readonly IReadOnlyDictionary<string, Attribute> Attributes;
    public readonly IReadOnlyDictionary<string, Uniform> Uniforms;

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

                if (!gl.GetProgramParameter<bool>(program, WebGL2RenderingContext.ShaderProgramParameter.LINK_STATUS))
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

        var attributes = new Dictionary<string, Attribute>();
        for (var i = 0; i < gl.GetProgramParameter<int>(program, WebGL2RenderingContext.ShaderProgramParameter.ACTIVE_ATTRIBUTES); i++)
        {
            var info = gl.GetActiveAttrib(program, i);
            var location = gl.GetAttribLocation(program, info.Name);
            attributes.Add(info.Name, new(info, location));
        }
        this.Attributes = attributes;

        var uniforms = new Dictionary<string, Uniform>();
        for (var i = 0; i < gl.GetProgramParameter<int>(program, WebGL2RenderingContext.ShaderProgramParameter.ACTIVE_UNIFORMS); i++)
        {
            var info = gl.GetActiveUniform(program, i);
            var location = gl.GetUniformLocation(program, info.Name);
            uniforms.Add(info.Name, new(this, info, location));
        }
        this.Uniforms = uniforms;
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

        if (!gl.GetShaderParameter<bool>(result, WebGL2RenderingContext.ShaderParameter.COMPILE_STATUS))
        {
            var log = gl.GetShaderInfoLog(result);
            gl.DeleteShader(result);
            throw new Exception($"error compiling shader: {log}");
        }

        return result;
    }
}
