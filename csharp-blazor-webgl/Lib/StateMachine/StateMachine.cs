﻿using BlazorExperiments.Lib.Dom;
using BlazorExperiments.Lib.WebGl;
using Microsoft.JSInterop;
using System.Drawing;

namespace BlazorExperiments.Lib.StateMachine;

public class StateMachine : IAsyncDisposable
{
    public readonly IJSRuntime JS;

    private readonly Canvas canvas;
    private readonly WebGL2RenderingContext gl;

    private IState? currentState;

    private TimeSpan? lastAnim;

    private Size? size;

    private readonly Dictionary<MouseButton, bool> mouseButtonState;
    private readonly Dictionary<KeyboardKey, bool> keyState;

    private StateMachine(IJSRuntime js, Canvas canvas, WebGL2RenderingContext gl, IState initialState)
    {
        this.JS = js;
        this.canvas = canvas;
        this.gl = gl;
        currentState = initialState;

        mouseButtonState = new Dictionary<MouseButton, bool>();
        keyState = new Dictionary<KeyboardKey, bool>();
    }

    public async ValueTask DisposeAsync()
    {
        if (currentState != null)
        {
            await currentState.DeactivateAsync(this, gl);
            currentState = null;
        }
    }

    public static async Task<StateMachine> Create(IJSRuntime js, Canvas canvas, WebGL2RenderingContext gl, IState initialState)
    {
        var result = new StateMachine(js, canvas, gl, initialState);
        await initialState.ActivateAsync(result, gl);
        return result;
    }

    public bool IsPointerLocked
    {
        get => canvas.IsPointerLocked;
        set => canvas.IsPointerLocked = value;
    }

    public bool GetMouseButtonState(MouseButton button)
    {
        return mouseButtonState.GetValueOrDefault(button, false);
    }

    public bool GetKeyState(KeyboardKey key)
    {
        return keyState.GetValueOrDefault(key, false);
    }

    public async Task ResizeAsync(Size size)
    {
        this.size = size;
        if (currentState != null)
        {
            await PossibleSwitchTo(await currentState.ResizeAsync(this, gl, size));
        }
    }

    public async Task Anim(TimeSpan timeSpan)
    {
        if (currentState != null)
        {
            if (lastAnim != null)
            {
                var delta = timeSpan - lastAnim.Value;
                await PossibleSwitchTo(await currentState.UpdateAsync(this, gl, delta));
            }
            lastAnim = timeSpan;

            await currentState.RenderAsync(this, gl);
        }
    }

    public async Task MouseDown(MouseEvent e)
    {
        mouseButtonState[e.Button] = true;
        if (currentState != null)
        {
            await currentState.MouseDown(this, e);
        }
    }

    public async Task MouseUp(MouseEvent e)
    {
        mouseButtonState[e.Button] = false;
        if (currentState != null)
        {
            await currentState.MouseUp(this, e);
        }
    }

    public async Task MouseMove(MouseMoveEvent e)
    {
        if (currentState != null)
        {
            await currentState.MouseMove(this, e);
        }
    }

    public async Task KeyDown(KeyEvent e)
    {
        keyState[e.Key] = true;
        if (currentState != null)
        {
            await currentState.KeyDown(this, e);
        }
    }

    public async Task KeyUp(KeyEvent e)
    {
        keyState[e.Key] = false;
        if (currentState != null)
        {
            await currentState.KeyUp(this, e);
        }
    }

    private async Task PossibleSwitchTo(IState nextState)
    {
        if (nextState != currentState)
        {
            await nextState.ActivateAsync(this, gl);
            if (currentState != null)
            {
                await currentState.DeactivateAsync(this, gl);
            }
            currentState = nextState;

            if (size != null)
            {
                await PossibleSwitchTo(await currentState.ResizeAsync(this, gl, size.Value));
            }
        }
    }
}
