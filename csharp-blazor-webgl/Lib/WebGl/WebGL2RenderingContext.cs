using BlazorExperiments.Lib.Math;
using Microsoft.JSInterop;

namespace BlazorExperiments.Lib.WebGl;

public class WebGL2RenderingContext
{
    public record class Shader(IJSInProcessObjectReference ObjRef);

    public record class ShaderProgram(IJSInProcessObjectReference ObjRef);

    public record class ActiveInfo(IJSInProcessObjectReference ObjRef, IJSInProcessObjectReference ContextObjRef)
    {
        public string Name => ContextObjRef.Invoke<string>("getActiveInfoName", ObjRef);

        public ActiveInfoType Type => ContextObjRef.Invoke<ActiveInfoType>("getActiveInfoType", ObjRef);

        public int Size => ContextObjRef.Invoke<int>("getActiveInfoSize", ObjRef);
    }

    public record class UniformLocation(IJSInProcessObjectReference ObjRef);

    public record class Buffer(IJSInProcessObjectReference ObjRef);

    public record class Texture(IJSInProcessObjectReference ObjRef);

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
        ACTIVE_ATTRIBUTES = 0x8B89,
        ACTIVE_UNIFORMS = 0x8B86,
    }

    public enum BufferTarget
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

    public enum ActiveInfoType
    {
        FLOAT_VEC2 = 0x8B50,
        FLOAT_VEC3 = 0x8B51,
        FLOAT_VEC4 = 0x8B52,
        INT_VEC2 = 0x8B53,
        INT_VEC3 = 0x8B54,
        INT_VEC4 = 0x8B55,
        BOOL = 0x8B56,
        BOOL_VEC2 = 0x8B57,
        BOOL_VEC3 = 0x8B58,
        BOOL_VEC4 = 0x8B59,
        FLOAT_MAT2 = 0x8B5A,
        FLOAT_MAT3 = 0x8B5B,
        FLOAT_MAT4 = 0x8B5C,
        SAMPLER_2D = 0x8B5E,
        SAMPLER_CUBE = 0x8B60,
    }

    public enum TextureTarget
    {
        TEXTURE_2D = 0x0DE1,
        TEXTURE_CUBE_MAP_POSITIVE_X = 0x8515,
        TEXTURE_CUBE_MAP_NEGATIVE_X = 0x8516,
        TEXTURE_CUBE_MAP_POSITIVE_Y = 0x8517,
        TEXTURE_CUBE_MAP_NEGATIVE_Y = 0x8518,
        TEXTURE_CUBE_MAP_POSITIVE_Z = 0x8519,
        TEXTURE_CUBE_MAP_NEGATIVE_Z = 0x851A,
    }

    public enum TextureInternalFormat
    {
        RGBA = 0x1908,
        RGB = 0x1907,
        LUMINANCE_ALPHA = 0x190A,
        LUMINANCE = 0x1909,
        ALPHA = 0x1906,
    }

    public enum TextureFormat
    {
        RGBA = 0x1908,
        RGB = 0x1907,
        LUMINANCE_ALPHA = 0x190A,
        LUMINANCE = 0x1909,
        ALPHA = 0x1906,
    }

    public enum TextureDataType
    {
        UNSIGNED_BYTE = 0x1401,
        BYTE = 0x1400,
        UNSIGNED_SHORT = 0x1403,
        SHORT = 0x1402,
        UNSIGNED_INT = 0x1405,
        INT = 0x1404,
        UNSIGNED_SHORT_5_6_5 = 0x8363,
        UNSIGNED_SHORT_4_4_4_4 = 0x8033,
        UNSIGNED_SHORT_5_5_5_1 = 0x8034,
        UNSIGNED_INT_24_8_WEBGL = 0x84FA,
        UNSIGNED_INT_2_10_10_10_REV = 0x8368,
        UNSIGNED_INT_10F_11F_11F_REV = 0x8C3B,
        UNSIGNED_INT_5_9_9_9_REV = 0x8C3E,
        UNSIGNED_INT_24_8 = 0x84FA,
        FLOAT = 0x1406,
        HALF_FLOAT = 0x140B,
        HALF_FLOAT_OES = 0x8D61,
        FLOAT_32_UNSIGNED_INT_24_8_REV = 0x8DAD,
    }

    public enum ActiveTextureIndex
    {
        TEXTURE0 = 0x84C0,
        TEXTURE1 = 0x84C1,
        TEXTURE2 = 0x84C2,
        TEXTURE3 = 0x84C3,
        TEXTURE4 = 0x84C4,
        TEXTURE5 = 0x84C5,
        TEXTURE6 = 0x84C6,
        TEXTURE7 = 0x84C7,
        TEXTURE8 = 0x84C8,
        TEXTURE9 = 0x84C9,
        TEXTURE10 = 0x84CA,
        TEXTURE11 = 0x84CB,
        TEXTURE12 = 0x84CC,
        TEXTURE13 = 0x84CD,
        TEXTURE14 = 0x84CE,
        TEXTURE15 = 0x84CF,
        TEXTURE16 = 0x84D0,
        TEXTURE17 = 0x84D1,
        TEXTURE18 = 0x84D2,
        TEXTURE19 = 0x84D3,
        TEXTURE20 = 0x84D4,
        TEXTURE21 = 0x84D5,
        TEXTURE22 = 0x84D6,
        TEXTURE23 = 0x84D7,
        TEXTURE24 = 0x84D8,
        TEXTURE25 = 0x84D9,
        TEXTURE26 = 0x84DA,
        TEXTURE27 = 0x84DB,
        TEXTURE28 = 0x84DC,
        TEXTURE29 = 0x84DD,
        TEXTURE30 = 0x84DE,
        TEXTURE31 = 0x84DF,
    }

    public enum TextureParameter
    {
        TEXTURE_MAG_FILTER = 0x2800,
        TEXTURE_MIN_FILTER = 0x2801,
        TEXTURE_WRAP_S = 0x2802,
        TEXTURE_WRAP_T = 0x2803,
    }

    public enum TextureMagFilter
    {
        LINEAR = 0x2601,
        NEAREST = 0x2600,
    }

    public enum TextureMinFilter
    {
        LINEAR = 0x2601,
        NEAREST = 0x2600,
        NEAREST_MIPMAP_NEAREST = 0x2700,
        LINEAR_MIPMAP_NEAREST = 0x2701,
        NEAREST_MIPMAP_LINEAR = 0x2702,
        LINEAR_MIPMAP_LINEAR = 0x2703,
    }

    public enum TextureWrap
    {
        REPEAT = 0x2901,
        CLAMP_TO_EDGE = 0x812F,
        MIRRORED_REPEAT = 0x8370,
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

    public void ClearColor(ColorRGBA<double> color)
    {
        ClearColor(color.Red, color.Green, color.Blue, color.Alpha);
    }

    public void Clear(ClearBuffer bits)
    {
        objRef.InvokeVoid("clear", bits);
    }

    public Shader CreateShader(ShaderType type)
    {
        return new(objRef.Invoke<IJSInProcessObjectReference>("createShader", type));
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
        return new(objRef.Invoke<IJSInProcessObjectReference>("createProgram"));
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

    public ActiveInfo GetActiveAttrib(ShaderProgram program, int index)
    {
        return new(objRef.Invoke<IJSInProcessObjectReference>("getActiveAttrib", program.ObjRef, index), objRef);
    }

    public int GetAttribLocation(ShaderProgram program, string name)
    {
        return objRef.Invoke<int>("getAttribLocation", program.ObjRef, name);
    }

    public ActiveInfo GetActiveUniform(ShaderProgram program, int index)
    {
        return new ActiveInfo(objRef.Invoke<IJSInProcessObjectReference>("getActiveUniform", program.ObjRef, index), objRef);
    }

    public UniformLocation GetUniformLocation(ShaderProgram program, string name)
    {
        return new(objRef.Invoke<IJSInProcessObjectReference>("getUniformLocation", program.ObjRef, name));
    }

    public void Uniform1i(UniformLocation location, int value)
    {
        objRef.InvokeVoid("uniform1i", location.ObjRef, value);
    }

    // TODO more uniforms uniform[1234][fi][v]()

    // TODO uniformMatrix[23]fv

    public void UniformMatrix4fv(UniformLocation location, bool transpose, float[] value)
    {
        objRef.InvokeVoid("uniformMatrix4fv", location.ObjRef, transpose, value);
    }

    public void UseProgram(ShaderProgram? program)
    {
        objRef.InvokeVoid("useProgram", program?.ObjRef);
    }

    public Buffer CreateBuffer()
    {
        return new(objRef.Invoke<IJSInProcessObjectReference>("createBuffer"));
    }

    public void DeleteBuffer(Buffer buffer)
    {
        objRef.InvokeVoid("deleteBuffer", buffer.ObjRef);
    }

    public void BindBuffer(BufferTarget target, Buffer? buffer)
    {
        objRef.InvokeVoid("bindBuffer", target, buffer?.ObjRef);
    }

    public void BufferData(BufferTarget target, int size, BufferUsage usage)
    {
        objRef.InvokeVoid("bufferData", target, size, usage);
    }

    public void BufferData(BufferTarget target, byte[] data, BufferUsage usage)
    {
        objRef.InvokeVoid("bufferData_uint8", target, data, usage);
    }

    public void BufferData(BufferTarget target, float[] data, BufferUsage usage)
    {
        objRef.InvokeVoid("bufferData_float32", target, data, usage);
    }

    public void BufferData(BufferTarget target, double[] data, BufferUsage usage)
    {
        objRef.InvokeVoid("bufferData_float64", target, data, usage);
    }

    public void BufferSubData(BufferTarget target, int offset, byte[] data)
    {
        objRef.InvokeVoid("bufferSubData_uint8", target, offset, data);
    }

    public void BufferSubData(BufferTarget target, int offset, float[] data)
    {
        objRef.InvokeVoid("bufferSubData_float32", target, offset, data);
    }

    public void BufferSubData(BufferTarget target, int offset, double[] data)
    {
        objRef.InvokeVoid("bufferSubData_float64", target, offset, data);
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

    public Texture CreateTexture()
    {
        return new(objRef.Invoke<IJSInProcessObjectReference>("createTexture"));
    }

    public void DeleteTexture(Texture texture)
    {
        objRef.InvokeVoid("deleteTexture", texture.ObjRef);
    }

    public void ActiveTexture(ActiveTextureIndex texture)
    {
        objRef.InvokeVoid("activeTexture", texture);
    }

    public void BindTexture(TextureTarget target, Texture? texture)
    {
        objRef.InvokeVoid("bindTexture", target, texture?.ObjRef);
    }

    public void GenerateMipmap(TextureTarget target)
    {
        objRef.InvokeVoid("generateMipmap", target);
    }

    public void TexParameter(TextureTarget target, TextureParameter pname, TextureMagFilter param)
    {
        objRef.InvokeVoid("texParameteri", target, pname, param);
    }

    public void TexParameter(TextureTarget target, TextureParameter pname, TextureMinFilter param)
    {
        objRef.InvokeVoid("texParameteri", target, pname, param);
    }

    public void TexParameter(TextureTarget target, TextureParameter pname, TextureWrap param)
    {
        objRef.InvokeVoid("texParameteri", target, pname, param);
    }

    public void TexImage2D(TextureTarget target, int level, TextureInternalFormat internalFormat, int width, int height, int border, TextureFormat format, TextureDataType type, byte[] pixels)
    {
        objRef.InvokeVoid("texImage2D", target, level, internalFormat, width, height, border, format, type, pixels);
    }
}
