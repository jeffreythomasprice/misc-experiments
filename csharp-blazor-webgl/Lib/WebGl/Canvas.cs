using BlazorExperiments.Lib.StateMachine;
using Microsoft.AspNetCore.Components;
using Microsoft.JSInterop;

namespace BlazorExperiments.Lib.WebGl;

public class Canvas : IAsyncDisposable
{
    private DotNetObjectReference<Canvas>? thisRef;
    private StateMachine.StateMachine? stateMachine;

    public static async Task<Canvas> Create(IJSRuntime js, ElementReference canvas, IState initialState)
    {
        var jsInProcess = (IJSInProcessRuntime)js;
        var module = await jsInProcess.InvokeAsync<IJSInProcessObjectReference>("import", "./_content/Lib/Canvas.js");
        var result = new Canvas();
        var gl = new WebGL2RenderingContext(module.Invoke<IJSInProcessObjectReference>("init", result.thisRef, canvas));
        result.stateMachine = await StateMachine.StateMachine.Create(gl, initialState);
        return result;
    }

    private Canvas()
    {
        thisRef = DotNetObjectReference.Create(this);
    }
    public ValueTask DisposeAsync()
    {
        thisRef?.Dispose();
        return ValueTask.CompletedTask;
    }

    [JSInvokable]
    public void Resize(int width, int height)
    {
        stateMachine?.ResizeAsync(width, height);
    }

    [JSInvokable]
    public void Anim(double time)
    {
        stateMachine?.Anim(TimeSpan.FromMilliseconds(time));
    }
}
