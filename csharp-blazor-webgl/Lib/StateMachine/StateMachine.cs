using BlazorExperiments.Lib.WebGl;
using System.Drawing;

namespace BlazorExperiments.Lib.StateMachine;

public class StateMachine : IAsyncDisposable
{
    private readonly Canvas canvas;
    private readonly WebGL2RenderingContext gl;
    private IState? currentState;
    private TimeSpan? lastAnim;
    private Size? size;

    private StateMachine(Canvas canvas, WebGL2RenderingContext gl, IState initialState)
    {
        this.canvas = canvas;
        this.gl = gl;
        currentState = initialState;
    }

    public async ValueTask DisposeAsync()
    {
        if (currentState != null)
        {
            await currentState.DeactivateAsync(this, gl);
            currentState = null;
        }
    }

    public static async Task<StateMachine> Create(Canvas canvas, WebGL2RenderingContext gl, IState initialState)
    {
        var result = new StateMachine(canvas, gl, initialState);
        await initialState.ActivateAsync(result, gl);
        return result;
    }

    public bool IsPointerLocked
    {
        get => canvas.IsPointerLocked;
        set => canvas.IsPointerLocked = value;
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
        // TODO keep track of button states
        if (currentState != null)
        {
            await currentState.MouseDown(this, e);
        }
    }

    public async Task MouseUp(MouseEvent e)
    {
        // TODO keep track of button states
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
        // TODO keep track of key states
        if (currentState != null)
        {
            await currentState.KeyDown(this, e);
        }
    }

    public async Task KeyUp(KeyEvent e)
    {
        // TODO keep track of key states
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
