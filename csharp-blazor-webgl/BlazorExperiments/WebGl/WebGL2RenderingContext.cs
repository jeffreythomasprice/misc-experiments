using Microsoft.JSInterop;

namespace BlazorExperiments.WebGl;

public class WebGL2RenderingContext
{
    public record class Shader(IJSInProcessObjectReference ObjRef);
    public record class ShaderProgram(IJSInProcessObjectReference ObjRef);
    public record class Buffer(IJSInProcessObjectReference ObjRef);

    public const int COLOR_BUFFER_BIT = 0x00004000;

    public const int FRAGMENT_SHADER = 0x8B30;
    public const int VERTEX_SHADER = 0x8B31;

    public const int COMPILE_STATUS = 0x8B81;
    public const int LINK_STATUS = 0x8B82;

    public const int ARRAY_BUFFER = 0x8892;
    public const int ELEMENT_ARRAY_BUFFER = 0x8893;

    public const int STATIC_DRAW = 0x88E4;
    public const int STREAM_DRAW = 0x88E0;
    public const int DYNAMIC_DRAW = 0x88E8;
    public const int STREAM_READ = 0x88E1;
    public const int STREAM_COPY = 0x88E2;
    public const int STATIC_READ = 0x88E5;
    public const int STATIC_COPY = 0x88E6;
    public const int DYNAMIC_READ = 0x88E9;
    public const int DYNAMIC_COPY = 0x88EA;

    public const int BYTE = 0x1400;
    public const int UNSIGNED_BYTE = 0x1401;
    public const int SHORT = 0x1402;
    public const int UNSIGNED_SHORT = 0x1403;
    public const int INT = 0x1404;
    public const int UNSIGNED_INT = 0x1405;
    public const int FLOAT = 0x1406;

    public const int POINTS = 0x0000;
    public const int LINES = 0x0001;
    public const int LINE_LOOP = 0x0002;
    public const int LINE_STRIP = 0x0003;
    public const int TRIANGLES = 0x0004;
    public const int TRIANGLE_STRIP = 0x0005;
    public const int TRIANGLE_FAN = 0x0006;

    private readonly IJSInProcessObjectReference objRef;

    public WebGL2RenderingContext(IJSInProcessObjectReference objRef)
    {
        this.objRef = objRef;
    }

    public void Viewport(int x, int y, int width, int height)
    {
        objRef.InvokeVoid("viewport", x, y, width, height);
    }

    public void ClearColor(double red, double green, double blue, double alpha)
    {
        objRef.InvokeVoid("clearColor", red, green, blue, alpha);
    }

    public void Clear(int bits)
    {
        objRef.InvokeVoid("clear", bits);
    }

    public Shader CreateShader(int type)
    {
        return new Shader(objRef.Invoke<IJSInProcessObjectReference>("createShader", type));
    }

    public void DeleteShader(Shader shader)
    {
        objRef.InvokeVoid("deleteShader", shader.ObjRef);
    }

    public void ShaderSource(Shader shader, string source)
    {
        objRef.InvokeVoid("shaderSource", shader.ObjRef, source);
    }

    public void CompileShader(Shader shader)
    {
        objRef.InvokeVoid("compileShader", shader.ObjRef);
    }

    public T GetShaderParameter<T>(Shader shader, int pname)
    {
        return objRef.Invoke<T>("getShaderParameter", shader.ObjRef, pname);
    }

    public string GetShaderInfoLog(Shader program)
    {
        return objRef.Invoke<string>("getShaderInfoLog", program.ObjRef);
    }

    public ShaderProgram CreateProgram()
    {
        return new ShaderProgram(objRef.Invoke<IJSInProcessObjectReference>("createProgram"));
    }

    public void DeleteProgram(ShaderProgram program)
    {
        objRef.InvokeVoid("deleteProgram", program.ObjRef);
    }

    public void AttachShader(ShaderProgram program, Shader shader)
    {
        objRef.InvokeVoid("attachShader", program.ObjRef, shader.ObjRef);
    }

    public void DetachShader(ShaderProgram program, Shader shader)
    {
        objRef.InvokeVoid("detachShader", program.ObjRef, shader.ObjRef);
    }

    public void LinkProgram(ShaderProgram program)
    {
        objRef.InvokeVoid("linkProgram", program.ObjRef);
    }

    public T GetProgramParameter<T>(ShaderProgram program, int pname)
    {
        return objRef.Invoke<T>("getProgramParameter", program.ObjRef, pname);
    }

    public string GetProgramInfoLog(ShaderProgram program)
    {
        return objRef.Invoke<string>("getProgramInfoLog", program.ObjRef);
    }

    public int GetAttribLocation(ShaderProgram program, string name)
    {
        return objRef.Invoke<int>("getAttribLocation", program.ObjRef, name);
    }

    // TODO various shader methods for getting attributes and uniforms

    public void UseProgram(ShaderProgram? program)
    {
        objRef.InvokeVoid("useProgram", program?.ObjRef);
    }

    public Buffer CreateBuffer()
    {
        return new Buffer(objRef.Invoke<IJSInProcessObjectReference>("createBuffer"));
    }

    public void DeleteBuffer(Buffer buffer)
    {
        objRef.InvokeVoid("deleteBuffer", buffer.ObjRef);
    }

    public void BindBuffer(int type, Buffer? buffer)
    {
        objRef.InvokeVoid("bindBuffer", type, buffer?.ObjRef);
    }

    // TODO BufferData with size
    // TODO BufferData with byte[]

    public void BufferData(int type, float[] data, int usage)
    {
        objRef.InvokeVoid("bufferData_float32", type, data, usage);
    }

    // TODO BufferData with double[]

    public void EnableVertexAttribArray(int i)
    {
        objRef.InvokeVoid("enableVertexAttribArray", i);
    }

    public void DisableVertexAttribArray(int i)
    {
        objRef.InvokeVoid("disableVertexAttribArray", i);
    }

    public void VertexAttribPointer(int index, int size, int type, bool normalized, int stride, int offset)
    {
        objRef.InvokeVoid("vertexAttribPointer", index, size, type, normalized, stride, offset);
    }

    public void DrawArrays(int mode, int first, int count)
    {
        objRef.InvokeVoid("drawArrays", mode, first, count);
    }

    // TODO drawElements
}
