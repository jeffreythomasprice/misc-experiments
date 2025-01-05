using Microsoft.JSInterop;

namespace BlazorExperiments.Lib.WebGl;

public class WebGL2RenderingContext
{
    public record class Shader(IJSInProcessObjectReference ObjRef);
    public record class ShaderProgram(IJSInProcessObjectReference ObjRef);
    public record class Buffer(IJSInProcessObjectReference ObjRef);

    public enum ClearBuffer
    {
        COLOR_BUFFER_BIT = 0x00004000,
    }

    public enum ShaderType
    {
        FRAGMENT_SHADER = 0x8B30,
        VERTEX_SHADER = 0x8B31,
    }

    public enum ShaderParameter
    {
        COMPILE_STATUS = 0x8B81,
    }

    public enum ShaderProgramParameter
    {
        LINK_STATUS = 0x8B82,
    }

    public enum BufferType
    {
        ARRAY_BUFFER = 0x8892,
        ELEMENT_ARRAY_BUFFER = 0x8893,
    }

    public enum BufferUsage
    {
        STATIC_DRAW = 0x88E4,
        STREAM_DRAW = 0x88E0,
        DYNAMIC_DRAW = 0x88E8,
        STREAM_READ = 0x88E1,
        STREAM_COPY = 0x88E2,
        STATIC_READ = 0x88E5,
        STATIC_COPY = 0x88E6,
        DYNAMIC_READ = 0x88E9,
        DYNAMIC_COPY = 0x88EA,
    }

    public enum DataType
    {
        BYTE = 0x1400,
        UNSIGNED_BYTE = 0x1401,
        SHORT = 0x1402,
        UNSIGNED_SHORT = 0x1403,
        INT = 0x1404,
        UNSIGNED_INT = 0x1405,
        FLOAT = 0x1406,
    }

    public enum DrawMode
    {
        POINTS = 0x0000,
        LINES = 0x0001,
        LINE_LOOP = 0x0002,
        LINE_STRIP = 0x0003,
        TRIANGLES = 0x0004,
        TRIANGLE_STRIP = 0x0005,
        TRIANGLE_FAN = 0x0006,
    }

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

    public void Clear(ClearBuffer bits)
    {
        objRef.InvokeVoid("clear", bits);
    }

    public Shader CreateShader(ShaderType type)
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

    public T GetShaderParameter<T>(Shader shader, ShaderParameter pname)
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

    public T GetProgramParameter<T>(ShaderProgram program, ShaderProgramParameter pname)
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

    public void BindBuffer(BufferType type, Buffer? buffer)
    {
        objRef.InvokeVoid("bindBuffer", type, buffer?.ObjRef);
    }

    public void BufferData(BufferType type, int size, BufferUsage usage)
    {
        objRef.InvokeVoid("bufferData", type, size, usage);
    }

    // TODO BufferData with byte[]

    public void BufferData(BufferType type, float[] data, BufferUsage usage)
    {
        objRef.InvokeVoid("bufferData_float32", type, data, usage);
    }

    // TODO BufferData with double[]

    public void BufferSubData(BufferType type, int offset, byte[] data)
    {
        objRef.InvokeVoid("bufferSubData_uint8", type, offset, data);
    }

    public void BufferSubData(BufferType type, int offset, float[] data)
    {
        objRef.InvokeVoid("bufferSubData_float32", type, offset, data);
    }

    public void BufferSubData(BufferType type, int offset, double[] data)
    {
        objRef.InvokeVoid("bufferSubData_float64", type, offset, data);
    }

    public void EnableVertexAttribArray(int i)
    {
        objRef.InvokeVoid("enableVertexAttribArray", i);
    }

    public void DisableVertexAttribArray(int i)
    {
        objRef.InvokeVoid("disableVertexAttribArray", i);
    }

    public void VertexAttribPointer(int index, int size, DataType type, bool normalized, int stride, int offset)
    {
        objRef.InvokeVoid("vertexAttribPointer", index, size, type, normalized, stride, offset);
    }

    public void DrawArrays(DrawMode mode, int first, int count)
    {
        objRef.InvokeVoid("drawArrays", mode, first, count);
    }

    public void DrawElements(DrawMode mode, int count, DataType type, int offset)
    {
        objRef.InvokeVoid("drawElements", mode, count, type, offset);
    }
}
