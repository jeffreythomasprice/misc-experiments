using BlazorExperiments.Lib.WebGl;
using Microsoft.AspNetCore.Components;
using Microsoft.JSInterop;

namespace BlazorExperiments.Lib.StateMachine;

public class Canvas : IAsyncDisposable
{
    private DotNetObjectReference<Canvas>? thisRef;
    private IJSInProcessObjectReference? context;
    private StateMachine? stateMachine;

    public static async Task<Canvas> Create(IJSRuntime js, ElementReference canvas, IState initialState)
    {
        var result = new Canvas();

        // js init
        var jsInProcess = (IJSInProcessRuntime)js;
        var module = await jsInProcess.InvokeAsync<IJSInProcessObjectReference>("import", "./_content/Lib/Canvas.js");
        result.context = module.Invoke<IJSInProcessObjectReference>("init", result.thisRef, canvas);

        // state machine init
        result.stateMachine = await StateMachine.Create(result, new WebGL2RenderingContext(result.context), initialState);

        // initial resize event so we know the initial size of the screen
        result.context.InvokeVoid("resize");

        return result;
    }

    private Canvas()
    {
        thisRef = DotNetObjectReference.Create(this);
    }

    public ValueTask DisposeAsync()
    {
        thisRef?.Dispose();
        thisRef = null;
        return ValueTask.CompletedTask;
    }

    public bool IsPointerLocked
    {
        get => context?.Invoke<Boolean>("getIsPointerLocked") ?? false;
        set
        {
            context?.InvokeVoid("setIsPointerLocked", value);
        }
    }

    [JSInvokable]
    public void Resize(int width, int height)
    {
        stateMachine?.ResizeAsync(new(width, height));
    }

    [JSInvokable]
    public void Anim(double time)
    {
        stateMachine?.Anim(TimeSpan.FromMilliseconds(time));
    }

    [JSInvokable]
    public void MouseDown(int button, int x, int y)
    {
        var e = new MouseEvent(button, new(x, y));
        stateMachine?.MouseDown(e);
    }

    [JSInvokable]
    public void MouseUp(int button, int x, int y)
    {
        var e = new MouseEvent(button, new(x, y));
        stateMachine?.MouseUp(e);
    }

    [JSInvokable]
    public void MouseMove(int x, int y, int movementX, int movementY)
    {
        var e = new MouseMoveEvent(new(x, y), new(movementX, movementY));
        stateMachine?.MouseMove(e);
    }

    [JSInvokable]
    public void KeyDown(string key, string code)
    {
        var e = new KeyEvent(key, code);
        stateMachine?.KeyDown(e);
    }

    [JSInvokable]
    public void KeyUp(string key, string code)
    {
        var e = new KeyEvent(key, code);
        stateMachine?.KeyUp(e);
    }
}
