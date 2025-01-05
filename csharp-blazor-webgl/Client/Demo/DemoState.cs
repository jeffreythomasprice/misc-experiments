using BlazorExperiments.Lib.StateMachine;
using BlazorExperiments.Lib.WebGl;
using System.Drawing;
using System.Runtime.InteropServices;

namespace BlazorExperiments.Client.Demo;

public class DemoState : IState
{
    private struct Vertex
    {
        public float X;
        public float Y;
        public float Red;
        public float Green;
        public float Blue;
        public float Alpha;

        public Vertex(float x, float y, float red, float green, float blue, float alpha)
        {
            this.X = x;
            this.Y = y;
            this.Red = red;
            this.Green = green;
            this.Blue = blue;
            this.Alpha = alpha;
        }
    }

    private readonly Shader shader;
    private readonly Buffer<Vertex> arrayBuffer;
    private readonly Buffer<ushort> elementArrayBuffer;

    public static IState Create()
    {
        return new WaitForPendingTaskState(
            new SolidColorBackgroundState(0.75, 0.5, 0.25, 1.0),
            (gl) =>
            {
                var shader = new Shader(
                    gl,
                    """
                    attribute vec2 positionAttribute;
                    attribute vec4 colorAttribute;

                    varying vec4 colorVarying;

                    void main() {
                        gl_Position = vec4(positionAttribute, 0, 1);
                        colorVarying = colorAttribute;
                    }
                    """,
                        """
                    precision mediump float;
                
                    varying vec4 colorVarying;

                    void main() {
                        gl_FragColor = colorVarying;
                    }
                    """
                );

                var arrayBuffer = new Buffer<Vertex>(gl, WebGL2RenderingContext.BufferType.ARRAY_BUFFER, WebGL2RenderingContext.BufferUsage.STATIC_DRAW)
                {
                    new(-0.5f, -0.5f, 1.0f, 0.0f, 0.0f, 1.0f),
                    new(0.5f, -0.5f, 0.0f, 1.0f, 0.0f, 1.0f),
                    new(0.5f, 0.5f, 0.0f, 0.0f, 1.0f, 1.0f),
                    new(-0.5f, 0.5f, 0.0f, 1.0f, 1.0f, 1.0f),
                };

                var elementArrayBuffer = new Buffer<ushort>(gl, WebGL2RenderingContext.BufferType.ELEMENT_ARRAY_BUFFER, WebGL2RenderingContext.BufferUsage.STATIC_DRAW)
                {
                    0,1,2,
                    2,3,0,
                };

                return Task.FromResult<IState>(new DemoState(shader, arrayBuffer, elementArrayBuffer));
            }
        );
    }

    private DemoState(Shader shader, Buffer<Vertex> arrayBuffer, Buffer<ushort> elementArrayBuffer)
    {
        this.shader = shader;
        this.arrayBuffer = arrayBuffer;
        this.elementArrayBuffer = elementArrayBuffer;
    }

    public override Task<IState> ResizeAsync(WebGL2RenderingContext gl, Size size)
    {
        gl.Viewport(0, 0, size.Width, size.Height);

        return Task.FromResult<IState>(this);
    }

    public override Task RenderAsync(WebGL2RenderingContext gl)
    {
        gl.ClearColor(0.25, 0.5, 0.75, 1.0);
        gl.Clear(WebGL2RenderingContext.ClearBuffer.COLOR_BUFFER_BIT);

        shader.UseProgram();

        arrayBuffer.Bind();
        elementArrayBuffer.Bind();

        var positionAttribute = shader.GetAttribLocation("positionAttribute");
        gl.EnableVertexAttribArray(positionAttribute);
        gl.VertexAttribPointer(
            positionAttribute,
            2,
            WebGL2RenderingContext.DataType.FLOAT,
            false,
            Marshal.SizeOf<Vertex>(),
            0
        );

        var colorAttribute = shader.GetAttribLocation("colorAttribute");
        gl.EnableVertexAttribArray(colorAttribute);
        gl.VertexAttribPointer(
            colorAttribute,
            4,
            WebGL2RenderingContext.DataType.FLOAT,
            false,
            Marshal.SizeOf<Vertex>(),
            (int)Marshal.OffsetOf<Vertex>(nameof(Vertex.Red))
        );

        gl.DrawElements(WebGL2RenderingContext.DrawMode.TRIANGLES, elementArrayBuffer.Count, WebGL2RenderingContext.DataType.UNSIGNED_SHORT, 0);

        gl.DisableVertexAttribArray(positionAttribute);
        gl.DisableVertexAttribArray(colorAttribute);

        gl.BindBuffer(WebGL2RenderingContext.BufferType.ARRAY_BUFFER, null);
        gl.BindBuffer(WebGL2RenderingContext.BufferType.ELEMENT_ARRAY_BUFFER, null);

        gl.UseProgram(null);

        return Task.CompletedTask;
    }
}
