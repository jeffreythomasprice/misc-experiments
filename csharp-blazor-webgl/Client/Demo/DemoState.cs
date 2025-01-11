using BlazorExperiments.Lib.Math;
using BlazorExperiments.Lib.StateMachine;
using BlazorExperiments.Lib.WebGl;
using System.Drawing;

namespace BlazorExperiments.Client.Demo;

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
    private readonly Texture texture;
    private readonly Buffer<Vertex> arrayBuffer;
    private readonly Buffer<ushort> elementArrayBuffer;

    private readonly BoundVertexAttributes<Vertex> vertexAttributes;
    private readonly Shader.Uniform projectionMatrixUniform;
    private readonly Shader.Uniform modelViewMatrixUniform;
    private readonly Shader.Uniform samplerUniform;

    private PerspectiveCamera<float> perspectiveCamera;

    private Degrees<float> rotation;

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

                var perspectiveCamera = new PerspectiveCamera<float>(
                    new(0, 0),
                    new Degrees<float>(60).Radians,
                    new(0, 0, 6),
                    new(0, 0, 0),
                    new(0, 1, 0)
                );

                return Task.FromResult<IState>(new DemoState(gl, shader, texture, arrayBuffer, elementArrayBuffer, perspectiveCamera));
            }
        );
    }

    private DemoState(WebGL2RenderingContext gl, Shader shader, Texture texture, Buffer<Vertex> arrayBuffer, Buffer<ushort> elementArrayBuffer, PerspectiveCamera<float> perspectiveCamera)
    {
        this.shader = shader;
        this.texture = texture;
        this.arrayBuffer = arrayBuffer;
        this.elementArrayBuffer = elementArrayBuffer;

        vertexAttributes = new BoundVertexAttributes<Vertex>(gl, shader);
        projectionMatrixUniform = shader.Uniforms["projectionMatrixUniform"];
        modelViewMatrixUniform = shader.Uniforms["modelViewMatrixUniform"];
        samplerUniform = shader.Uniforms["samplerUniform"];

        this.perspectiveCamera = perspectiveCamera;
    }

    public override Task<IState> ResizeAsync(StateMachine sm, WebGL2RenderingContext gl, Size size)
    {
        gl.Viewport(0, 0, size.Width, size.Height);

        perspectiveCamera.WindowSize = size;

        return Task.FromResult<IState>(this);
    }

    public override Task<IState> UpdateAsync(StateMachine sm, WebGL2RenderingContext gl, TimeSpan timeSpan)
    {
        float forward = 0;
        float strafe = 0;
        float up = 0;
        if (sm.GetKeyState(KeyboardKey.ArrowUp) || sm.GetKeyState(KeyboardKey.KeyW))
        {
            forward += 1.0f;
        }
        if (sm.GetKeyState(KeyboardKey.ArrowDown) || sm.GetKeyState(KeyboardKey.KeyS))
        {
            forward -= 1.0f;
        }
        if (sm.GetKeyState(KeyboardKey.ArrowLeft) || sm.GetKeyState(KeyboardKey.KeyA))
        {
            strafe -= 1.0f;
        }
        if (sm.GetKeyState(KeyboardKey.ArrowRight) || sm.GetKeyState(KeyboardKey.KeyD))
        {
            strafe += 1.0f;
        }
        if (sm.GetKeyState(KeyboardKey.Space))
        {
            up += 1.0f;
        }
        if (sm.GetKeyState(KeyboardKey.ShiftLeft))
        {
            up -= 1.0f;
        }
        perspectiveCamera.Move(forward, strafe, up);

        rotation = (rotation + new Degrees<float>(90.0f) * new Degrees<float>((float)timeSpan.TotalSeconds)) % new Degrees<float>(360);

        return Task.FromResult<IState>(this);
    }

    public override Task RenderAsync(StateMachine sm, WebGL2RenderingContext gl)
    {
        gl.ClearColor(System.Drawing.Color.SkyBlue.ToRGBA().ToDouble());
        gl.Clear(WebGL2RenderingContext.ClearBuffer.COLOR_BUFFER_BIT);

        shader.UseProgram();

        arrayBuffer.Bind();
        elementArrayBuffer.Bind();

        vertexAttributes.Enable();

        projectionMatrixUniform.Set(true, perspectiveCamera.ProjectionMatrix);
        modelViewMatrixUniform.Set(
            false,
            perspectiveCamera.ModelViewMatrix
        // TODO put rotation back
        //Matrix4<float>.CreateLookAt(
        //    new(0, 0, 6),
        //    new(0, 0, 0),
        //    new(0, 1, 0)
        //)
        //    .Rotate(new(0, 1, 0), rotation.Radians)
        );

        gl.ActiveTexture(WebGL2RenderingContext.ActiveTextureIndex.TEXTURE0);
        samplerUniform.Set(0);
        texture.Bind();

        gl.DrawElements(WebGL2RenderingContext.DrawMode.TRIANGLES, elementArrayBuffer.Count, WebGL2RenderingContext.DataType.UNSIGNED_SHORT, 0);

        vertexAttributes.Disable();

        gl.BindBuffer(WebGL2RenderingContext.BufferTarget.ARRAY_BUFFER, null);
        gl.BindBuffer(WebGL2RenderingContext.BufferTarget.ELEMENT_ARRAY_BUFFER, null);

        gl.BindTexture(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, null);

        gl.UseProgram(null);

        return Task.CompletedTask;
    }

    public override Task MouseUp(StateMachine sm, MouseEvent e)
    {
        if (e.Button == 0)
        {
            sm.IsPointerLocked = !sm.IsPointerLocked;
        }
        return Task.CompletedTask;
    }

    public override Task MouseMove(StateMachine sm, MouseMoveEvent e)
    {
        if (sm.IsPointerLocked)
        {
            perspectiveCamera.Turn(new(e.Movement.X, e.Movement.Y));
        }
        return Task.CompletedTask;
    }
}
