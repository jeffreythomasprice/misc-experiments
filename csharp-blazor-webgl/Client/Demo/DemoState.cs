using BlazorExperiments.Lib.Math;
using BlazorExperiments.Lib.StateMachine;
using BlazorExperiments.Lib.WebGl;
using System.Drawing;
using System.Runtime.InteropServices;

namespace BlazorExperiments.Client.Demo;

public class DemoState : IState
{
    private struct Vertex
    {
        public readonly Vector2<float> Position;
        public readonly Vector2<float> TextureCoordinate;

        public Vertex(Vector2<float> position, Vector2<float> textureCoordinate)
        {
            Position = position;
            TextureCoordinate = textureCoordinate;
        }
    }

    private readonly Shader shader;
    private readonly Texture texture;
    private readonly Buffer<Vertex> arrayBuffer;
    private readonly Buffer<ushort> elementArrayBuffer;

    private readonly Shader.Attribute positionAttribute;
    private readonly Shader.Attribute textureCoordinateAttribute;
    private readonly Shader.Uniform samplerUniform;

    public static IState Create()
    {
        return new WaitForPendingTaskState(
            new SolidColorBackgroundState(System.Drawing.Color.HotPink.ToRGBA().ToDouble()),
            (gl) =>
            {
                var shader = new Shader(
                    gl,
                    """
                    attribute vec2 positionAttribute;
                    attribute vec2 textureCoordinateAttribute;

                    varying vec2 textureCoordinateVarying;

                    void main() {
                        gl_Position = vec4(positionAttribute, 0, 1);
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

                var arrayBuffer = new Buffer<Vertex>(gl, WebGL2RenderingContext.BufferTarget.ARRAY_BUFFER, WebGL2RenderingContext.BufferUsage.STATIC_DRAW)
                {
                    new (new( -0.5f, -0.5f ), new(0,0)),
                    new (new( -0.5f, +0.5f ), new(0,1)),
                    new (new( +0.5f, +0.5f ), new(1,1)),
                    new (new( +0.5f, -0.5f ), new(1,0)),
                };

                var elementArrayBuffer = new Buffer<ushort>(gl, WebGL2RenderingContext.BufferTarget.ELEMENT_ARRAY_BUFFER, WebGL2RenderingContext.BufferUsage.STATIC_DRAW)
                {
                    0,1,2,
                    2,3,0,
                };

                return Task.FromResult<IState>(new DemoState(shader, texture, arrayBuffer, elementArrayBuffer));
            }
        );
    }

    private DemoState(Shader shader, Texture texture, Buffer<Vertex> arrayBuffer, Buffer<ushort> elementArrayBuffer)
    {
        this.shader = shader;
        this.texture = texture;
        this.arrayBuffer = arrayBuffer;
        this.elementArrayBuffer = elementArrayBuffer;

        positionAttribute = shader.Attributes["positionAttribute"];
        textureCoordinateAttribute = shader.Attributes["textureCoordinateAttribute"];
        samplerUniform = shader.Uniforms["samplerUniform"];
    }

    public override Task<IState> ResizeAsync(WebGL2RenderingContext gl, Size size)
    {
        gl.Viewport(0, 0, size.Width, size.Height);

        return Task.FromResult<IState>(this);
    }

    public override Task RenderAsync(WebGL2RenderingContext gl)
    {
        gl.ClearColor(System.Drawing.Color.SkyBlue.ToRGBA().ToDouble());
        gl.Clear(WebGL2RenderingContext.ClearBuffer.COLOR_BUFFER_BIT);

        shader.UseProgram();

        gl.ActiveTexture(WebGL2RenderingContext.ActiveTextureIndex.TEXTURE0);
        samplerUniform.Set(0);
        texture.Bind();

        arrayBuffer.Bind();
        elementArrayBuffer.Bind();

        gl.EnableVertexAttribArray(positionAttribute.Location);
        gl.VertexAttribPointer(
            positionAttribute.Location,
            2,
            WebGL2RenderingContext.DataType.FLOAT,
            false,
            Marshal.SizeOf<Vertex>(),
            (int)Marshal.OffsetOf<Vertex>(nameof(Vertex.Position))
        );

        gl.EnableVertexAttribArray(textureCoordinateAttribute.Location);
        gl.VertexAttribPointer(
            textureCoordinateAttribute.Location,
            2,
            WebGL2RenderingContext.DataType.FLOAT,
            false,
            Marshal.SizeOf<Vertex>(),
            (int)Marshal.OffsetOf<Vertex>(nameof(Vertex.TextureCoordinate))
        );

        gl.DrawElements(WebGL2RenderingContext.DrawMode.TRIANGLES, elementArrayBuffer.Count, WebGL2RenderingContext.DataType.UNSIGNED_SHORT, 0);

        gl.DisableVertexAttribArray(positionAttribute.Location);
        gl.DisableVertexAttribArray(textureCoordinateAttribute.Location);

        gl.BindBuffer(WebGL2RenderingContext.BufferTarget.ARRAY_BUFFER, null);
        gl.BindBuffer(WebGL2RenderingContext.BufferTarget.ELEMENT_ARRAY_BUFFER, null);

        gl.BindTexture(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, null);

        gl.UseProgram(null);

        return Task.CompletedTask;
    }
}
