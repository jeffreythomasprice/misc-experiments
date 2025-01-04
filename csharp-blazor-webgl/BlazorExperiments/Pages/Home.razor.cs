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

    private WebGL2RenderingContext? context;
    private Shader? shader;
    private WebGL2RenderingContext.Buffer? arrayBuffer;

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
            context = new WebGL2RenderingContext(module.Invoke<IJSInProcessObjectReference>("init", thisRef, Canvas));

            shader = new Shader(
                context,
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

            arrayBuffer = context.CreateBuffer();
            context.BindBuffer(WebGL2RenderingContext.ARRAY_BUFFER, arrayBuffer);
            context.BufferData(
                WebGL2RenderingContext.ARRAY_BUFFER,
                [
                    -0.5f, -0.5f,
                    0.5f, -0.5f,
                    0.0f, 0.5f,
                ],
                WebGL2RenderingContext.STATIC_DRAW
            );
            context.BindBuffer(WebGL2RenderingContext.ARRAY_BUFFER, null);
        }
    }

    [JSInvokable]
    public void Resize(int width, int height)
    {
        context?.Viewport(0, 0, width, height);
    }

    [JSInvokable]
    public void Anim(double time)
    {
        if (context == null || shader == null)
        {
            return;
        }

        context.ClearColor(0.25, 0.5, 0.75, 1.0);
        context.Clear(WebGL2RenderingContext.COLOR_BUFFER_BIT);

        shader.UseProgram();

        var positionAttribute = shader.GetAttribLocation("positionAttribute");

        context.BindBuffer(WebGL2RenderingContext.ARRAY_BUFFER, arrayBuffer);
        context.EnableVertexAttribArray(positionAttribute);
        context.VertexAttribPointer(
            positionAttribute,
            2,
            WebGL2RenderingContext.FLOAT,
            false,
            0,
            0
        );

        context.DrawArrays(WebGL2RenderingContext.TRIANGLES, 0, 3);

        context.DisableVertexAttribArray(positionAttribute);
        context.BindBuffer(WebGL2RenderingContext.ARRAY_BUFFER, null);

        context.UseProgram(null);
    }
}
