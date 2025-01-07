using BlazorExperiments.Lib.Math;
using BlazorExperiments.Lib.StateMachine;
using BlazorExperiments.Lib.WebGl;
using System.Drawing;
using System.Reflection;
using System.Runtime.InteropServices;

namespace BlazorExperiments.Client.Demo;

// TODO move me
[AttributeUsage(AttributeTargets.Field)]
public class VertexAttribute : Attribute
{
    // TODO make name optional, guess based on prop name
    public readonly string Name;
    public readonly int Size;
    public readonly WebGL2RenderingContext.DataType DataType;
    public readonly bool Normalized;

    public VertexAttribute(string name, int size, WebGL2RenderingContext.DataType dataType, bool normalized)
    {
        Name = name;
        Size = size;
        DataType = dataType;
        Normalized = normalized;
    }
}

// TODO move me
public class BoundVertexAttributes<T>
{
    private record class Item(
        VertexAttribute VertexAttribute,
        Shader.Attribute ShaderAttribute,
        int Offset
    );

    private readonly WebGL2RenderingContext gl;
    private readonly Shader shader;
    private readonly int stride;
    private readonly List<Item> items;

    public BoundVertexAttributes(WebGL2RenderingContext gl, Shader shader)
    {
        this.gl = gl;
        this.shader = shader;
        stride = Marshal.SizeOf<T>();

        Console.WriteLine($"TODO DoStuff {typeof(T)}");
        var items = new List<Item>();
        foreach (var f in typeof(T).GetFields())
        {
            Console.WriteLine($"TODO f = {f}");
            var vertexAttribute = f.GetCustomAttribute<VertexAttribute>();
            if (vertexAttribute != null)
            {
                Console.WriteLine($"TODO this has vertex attribute stuff, {vertexAttribute.Name}, {vertexAttribute.Size}, {vertexAttribute.DataType}, {vertexAttribute.Normalized}");
                var shaderAttribute = shader.Attributes[vertexAttribute.Name];
                if (shaderAttribute == null)
                {
                    throw new NullReferenceException($"no such shader attribute {vertexAttribute.Name}");
                }
                items.Add(new(vertexAttribute, shaderAttribute, (int)Marshal.OffsetOf<T>(f.Name)));
            }
        }
        this.items = items;
    }

    public void UseShaderAndEnableVertexAttributes()
    {
        foreach (var item in items)
        {
            gl.EnableVertexAttribArray(item.ShaderAttribute.Location);
            gl.VertexAttribPointer(
                item.ShaderAttribute.Location,
                item.VertexAttribute.Size,
                item.VertexAttribute.DataType,
                item.VertexAttribute.Normalized,
                stride,
                item.Offset
            );
        }
        shader.UseProgram();
    }

    public void DisableVertexAttributesAndUseNoShader()
    {
        foreach (var item in items)
        {
            gl.DisableVertexAttribArray(item.ShaderAttribute.Location);
        }
        gl.UseProgram(null);
    }
}

public class DemoState : IState
{
    private struct Vertex
    {
        [VertexAttribute("positionAttribute", 3, WebGL2RenderingContext.DataType.FLOAT, false)]
        public readonly Vector3<float> Position;
        [VertexAttribute("textureCoordinateAttribute", 2, WebGL2RenderingContext.DataType.FLOAT, false)]
        public readonly Vector2<float> TextureCoordinate;

        public Vertex(Vector3<float> position, Vector2<float> textureCoordinate)
        {
            Position = position;
            TextureCoordinate = textureCoordinate;
        }
    }

    private readonly Shader shader;
    private readonly BoundVertexAttributes<Vertex> vertexAttributes;
    private readonly Texture texture;
    private readonly Buffer<Vertex> arrayBuffer;
    private readonly Buffer<ushort> elementArrayBuffer;

    private readonly Shader.Attribute positionAttribute;
    private readonly Shader.Attribute textureCoordinateAttribute;
    private readonly Shader.Uniform projectionMatrixUniform;
    private readonly Shader.Uniform modelViewMatrixUniform;
    private readonly Shader.Uniform samplerUniform;

    private Matrix4<float> orthoMatrix;
    private Matrix4<float> perspectiveMatrix;

    private float rotation;

    public static IState Create()
    {
        return new WaitForPendingTaskState(
            new SolidColorBackgroundState(System.Drawing.Color.HotPink.ToRGBA().ToDouble()),
            (gl) =>
            {
                var shader = new Shader(
                    gl,
                    """
                    attribute vec3 positionAttribute;
                    attribute vec2 textureCoordinateAttribute;

                    uniform mat4 projectionMatrixUniform;
                    uniform mat4 modelViewMatrixUniform;

                    varying vec2 textureCoordinateVarying;

                    void main() {
                        gl_Position = projectionMatrixUniform * modelViewMatrixUniform * vec4(positionAttribute, 1);
                        textureCoordinateVarying = textureCoordinateAttribute;
                    }
                    """,
                        """
                    precision mediump float;
                
                    uniform sampler2D samplerUniform;

                    varying vec2 textureCoordinateVarying;

                    void main() {
                        gl_FragColor = texture2D(samplerUniform, textureCoordinateVarying);
                    }
                    """
                );

                var vertexAttributes = new BoundVertexAttributes<Vertex>(gl, shader);

                var textureSize = new Size(256, 256);
                var texturePixels = new ColorRGBA<byte>[textureSize.Width * textureSize.Height];
                for (var y = 0; y < textureSize.Height; y++)
                {
                    var b = (byte)((double)y / (double)textureSize.Height * 255.0);
                    for (var x = 0; x < textureSize.Width; x++)
                    {
                        var a = (byte)((double)x / (double)textureSize.Width * 255.0);
                        texturePixels[y * textureSize.Width + x] = new(a, b, a, 255);
                    }
                }
                var texture = new Texture(gl, textureSize, texturePixels);

                var textureAspectRatioHeight = 1.0f;
                var textureAspectRatioWidth = textureAspectRatioHeight * (float)textureSize.Width / (float)textureSize.Height;

                var arrayBuffer = new Buffer<Vertex>(gl, WebGL2RenderingContext.BufferTarget.ARRAY_BUFFER, WebGL2RenderingContext.BufferUsage.STATIC_DRAW)
                {
                    new (new(-textureAspectRatioWidth, -textureAspectRatioHeight, 0), new(0,0)),
                    new (new(-textureAspectRatioWidth, +textureAspectRatioHeight, 0), new(0,1)),
                    new (new(+textureAspectRatioWidth, +textureAspectRatioHeight, 0), new(1,1)),
                    new (new(+textureAspectRatioWidth, -textureAspectRatioHeight, 0), new(1,0)),
                };

                var elementArrayBuffer = new Buffer<ushort>(gl, WebGL2RenderingContext.BufferTarget.ELEMENT_ARRAY_BUFFER, WebGL2RenderingContext.BufferUsage.STATIC_DRAW)
                {
                    0,1,2,
                    2,3,0,
                };

                return Task.FromResult<IState>(new DemoState(shader, vertexAttributes, texture, arrayBuffer, elementArrayBuffer));
            }
        );
    }

    private DemoState(Shader shader, BoundVertexAttributes<Vertex> vertexAttributes, Texture texture, Buffer<Vertex> arrayBuffer, Buffer<ushort> elementArrayBuffer)
    {
        this.shader = shader;
        this.vertexAttributes = vertexAttributes;
        this.texture = texture;
        this.arrayBuffer = arrayBuffer;
        this.elementArrayBuffer = elementArrayBuffer;

        positionAttribute = shader.Attributes["positionAttribute"];
        textureCoordinateAttribute = shader.Attributes["textureCoordinateAttribute"];
        projectionMatrixUniform = shader.Uniforms["projectionMatrixUniform"];
        modelViewMatrixUniform = shader.Uniforms["modelViewMatrixUniform"];
        samplerUniform = shader.Uniforms["samplerUniform"];
    }

    public override Task<IState> ResizeAsync(WebGL2RenderingContext gl, Size size)
    {
        gl.Viewport(0, 0, size.Width, size.Height);

        orthoMatrix = Matrix4<float>.CreateOrtho(0, size.Width, size.Height, 0, -1, 1);
        perspectiveMatrix = Matrix4<float>.CreatePerspective(60f * MathF.PI / 180.0f, size.Width, size.Height, 0.01f, 1000.0f);

        return Task.FromResult<IState>(this);
    }

    public override Task<IState> UpdateAsync(WebGL2RenderingContext gl, TimeSpan timeSpan)
    {
        rotation = (rotation + 90.0f * MathF.PI / 180.0f * (float)timeSpan.TotalSeconds) % (MathF.PI * 2);

        return Task.FromResult<IState>(this);
    }

    public override Task RenderAsync(WebGL2RenderingContext gl)
    {
        gl.ClearColor(System.Drawing.Color.SkyBlue.ToRGBA().ToDouble());
        gl.Clear(WebGL2RenderingContext.ClearBuffer.COLOR_BUFFER_BIT);

        arrayBuffer.Bind();
        elementArrayBuffer.Bind();

        vertexAttributes.UseShaderAndEnableVertexAttributes();

        projectionMatrixUniform.Set(true, perspectiveMatrix);
        // TODO lookat camera
        modelViewMatrixUniform.Set(
            false,
            Matrix4<float>.Identity
                .Translate(new(0, 0, -6))
                .Rotate(new(0, 1, 0), rotation)
        );

        gl.ActiveTexture(WebGL2RenderingContext.ActiveTextureIndex.TEXTURE0);
        samplerUniform.Set(0);
        texture.Bind();

        gl.DrawElements(WebGL2RenderingContext.DrawMode.TRIANGLES, elementArrayBuffer.Count, WebGL2RenderingContext.DataType.UNSIGNED_SHORT, 0);

        vertexAttributes.DisableVertexAttributesAndUseNoShader();

        gl.BindBuffer(WebGL2RenderingContext.BufferTarget.ARRAY_BUFFER, null);
        gl.BindBuffer(WebGL2RenderingContext.BufferTarget.ELEMENT_ARRAY_BUFFER, null);

        gl.BindTexture(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, null);

        return Task.CompletedTask;
    }
}
