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
        public readonly Vector3<float> Position;
        public readonly Vector2<float> TextureCoordinate;

        public Vertex(Vector3<float> position, Vector2<float> textureCoordinate)
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

        shader.UseProgram();

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

        arrayBuffer.Bind();
        elementArrayBuffer.Bind();

        // TODO helpers for vertex attributes
        gl.EnableVertexAttribArray(positionAttribute.Location);
        gl.VertexAttribPointer(
            positionAttribute.Location,
            3,
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
