using BlazorExperiments.Lib.StateMachine;
using BlazorExperiments.Lib.WebGl;
using System.Runtime.InteropServices;

namespace BlazorExperiments.Client.Demo;

public class DemoState : IState
{
    private record struct Vertex(
        float X,
        float Y,
        float Red,
        float Green,
        float Blue,
        float Alpha
    );

    private Shader? shader;
    private Buffer<Vertex>? arrayBuffer;
    private Buffer<ushort>? elementArrayBuffer;

    public override Task ActivateAsync(WebGL2RenderingContext gl)
    {
        shader = new Shader(
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

        arrayBuffer = new Buffer<Vertex>(gl, WebGL2RenderingContext.BufferType.ARRAY_BUFFER, WebGL2RenderingContext.BufferUsage.STATIC_DRAW)
        {
            new(-0.5f, -0.5f, 1.0f, 0.0f, 0.0f, 1.0f),
            new(0.5f, -0.5f, 0.0f, 1.0f, 0.0f, 1.0f),
            new(0.5f, 0.5f, 0.0f, 0.0f, 1.0f, 1.0f),
            new(-0.5f, 0.5f, 0.0f, 1.0f, 1.0f, 1.0f),
        };
        elementArrayBuffer = new Buffer<ushort>(gl, WebGL2RenderingContext.BufferType.ELEMENT_ARRAY_BUFFER, WebGL2RenderingContext.BufferUsage.STATIC_DRAW)
        {
            0,1,2,
            2,3,0,
        };

        return Task.CompletedTask;
    }

    public override Task<IState> ResizeAsync(WebGL2RenderingContext gl, int width, int height)
    {
        gl.Viewport(0, 0, width, height);

        return Task.FromResult<IState>(this);
    }

    public override Task RenderAsync(WebGL2RenderingContext gl)
    {
        // TODO should be able to init before render
        if (shader == null || arrayBuffer == null || elementArrayBuffer == null)
        {
            return Task.CompletedTask;
        }

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
            // TODO how to use offsetof?
            8
        //(int)Marshal.OffsetOf<Vertex>("Red")
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
