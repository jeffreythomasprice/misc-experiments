using BlazorExperiments.WebGl;
using Microsoft.AspNetCore.Components;
using Microsoft.JSInterop;

namespace BlazorExperiments.Pages;

public partial class Home : ComponentBase
{
    [Inject]
    public required IJSRuntime JS { get; set; }

    private ElementReference Canvas;

    private DotNetObjectReference<Home>? thisRef;

    private WebGL2RenderingContext? gl;
    private Shader? shader;
    private Buffer<float>? arrayBuffer;

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

                void main() {
                	gl_Position = vec4(positionAttribute, 0, 1);
                }
                """,
                """
                void main() {
                	gl_FragColor = vec4(1, 1, 1, 1);
                }
                """
            );

            arrayBuffer = new Buffer<float>(gl, WebGL2RenderingContext.BufferType.ARRAY_BUFFER, WebGL2RenderingContext.BufferUsage.STATIC_DRAW) {
                -0.5f, -0.5f,
                0.5f, -0.5f,
                0.0f, 0.5f,
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
        if (gl == null || shader == null || arrayBuffer == null)
        {
            return;
        }

        gl.ClearColor(0.25, 0.5, 0.75, 1.0);
        gl.Clear(WebGL2RenderingContext.COLOR_BUFFER_BIT);

        shader.UseProgram();

        var positionAttribute = shader.GetAttribLocation("positionAttribute");

        arrayBuffer.Bind();
        gl.EnableVertexAttribArray(positionAttribute);
        gl.VertexAttribPointer(
            positionAttribute,
            2,
            WebGL2RenderingContext.FLOAT,
            false,
            0,
            0
        );

        gl.DrawArrays(WebGL2RenderingContext.TRIANGLES, 0, 3);

        gl.DisableVertexAttribArray(positionAttribute);
        gl.BindBuffer(WebGL2RenderingContext.BufferType.ARRAY_BUFFER, null);

        gl.UseProgram(null);
    }
}
