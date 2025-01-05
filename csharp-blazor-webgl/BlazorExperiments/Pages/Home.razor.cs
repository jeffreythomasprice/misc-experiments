using BlazorExperiments.WebGl;
using Microsoft.AspNetCore.Components;
using Microsoft.JSInterop;
using System.Runtime.InteropServices;

namespace BlazorExperiments.Pages;

public partial class Home : ComponentBase
{
    private record struct Vertex(
        float X,
        float Y,
        float Red,
        float Green,
        float Blue,
        float Alpha
    );

    [Inject]
    public required IJSRuntime JS { get; set; }

    private ElementReference Canvas;

    private DotNetObjectReference<Home>? thisRef;

    private WebGL2RenderingContext? gl;
    private Shader? shader;
    private Buffer<Vertex>? arrayBuffer;
    private Buffer<ushort>? elementArrayBuffer;

    public void Dispose()
    {
        thisRef?.Dispose();
    }

    protected override void OnInitialized()
    {
        thisRef = DotNetObjectReference.Create(this);
    }

    protected override async Task OnAfterRenderAsync(bool firstRender)
    {
        if (firstRender)
        {
            var jsInProcess = (IJSInProcessRuntime)JS;
            var module = await jsInProcess.InvokeAsync<IJSInProcessObjectReference>("import", "./js/experiment.js");
            gl = new WebGL2RenderingContext(module.Invoke<IJSInProcessObjectReference>("init", thisRef, Canvas));

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

            arrayBuffer = new Buffer<Vertex>(gl, WebGL2RenderingContext.BufferType.ARRAY_BUFFER, WebGL2RenderingContext.BufferUsage.STATIC_DRAW) {
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
        }
    }

    [JSInvokable]
    public void Resize(int width, int height)
    {
        gl?.Viewport(0, 0, width, height);
    }

    [JSInvokable]
    public void Anim(double time)
    {
        if (gl == null || shader == null || arrayBuffer == null || elementArrayBuffer == null)
        {
            return;
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
    }
}
