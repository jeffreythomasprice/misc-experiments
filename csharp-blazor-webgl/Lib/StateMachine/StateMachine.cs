using BlazorExperiments.Lib.WebGl;
using System.Drawing;

namespace BlazorExperiments.Lib.StateMachine;

public class StateMachine : IAsyncDisposable
{
    private readonly WebGL2RenderingContext gl;
    private IState? currentState;
    private TimeSpan? lastAnim;
    private Size? size;

    private StateMachine(WebGL2RenderingContext gl, IState initialState)
    {
        this.gl = gl;
        currentState = initialState;
    }

    public async ValueTask DisposeAsync()
    {
        if (currentState != null)
        {
            await currentState.DeactivateAsync(gl);
            currentState = null;
        }
    }

    public static async Task<StateMachine> Create(WebGL2RenderingContext gl, IState initialState)
    {
        await initialState.ActivateAsync(gl);
        return new StateMachine(gl, initialState);
    }

    public async Task ResizeAsync(Size size)
    {
        this.size = size;
        if (currentState != null)
        {
            await PossibleSwitchTo(await currentState.ResizeAsync(gl, size));
        }
    }

    public async Task Anim(TimeSpan timeSpan)
    {
        if (currentState != null)
        {
            if (lastAnim != null)
            {
                var delta = timeSpan - lastAnim.Value;
                await PossibleSwitchTo(await currentState.UpdateAsync(gl, delta));
            }
            lastAnim = timeSpan;

            await currentState.RenderAsync(gl);
        }
    }

    public async Task MouseDown(MouseEvent e)
    {
        // TODO keep track of button states
        if (currentState != null)
        {
            await currentState.MouseDown(e);
        }
    }

    public async Task MouseUp(MouseEvent e)
    {
        // TODO keep track of button states
        if (currentState != null)
        {
            await currentState.MouseUp(e);
        }
    }

    public async Task MouseMove(MouseMoveEvent e)
    {
        if (currentState != null)
        {
            await currentState.MouseMove(e);
        }
    }

    public async Task KeyDown(KeyEvent e)
    {
        // TODO keep track of key states
        if (currentState != null)
        {
            await currentState.KeyDown(e);
        }
    }

    public async Task KeyUp(KeyEvent e)
    {
        // TODO keep track of key states
        if (currentState != null)
        {
            await currentState.KeyUp(e);
        }
    }

    private async Task PossibleSwitchTo(IState nextState)
    {
        if (nextState != currentState)
        {
            await nextState.ActivateAsync(gl);
            if (currentState != null)
            {
                await currentState.DeactivateAsync(gl);
            }
            currentState = nextState;

            if (size != null)
            {
                await PossibleSwitchTo(await currentState.ResizeAsync(gl, size.Value));
            }
        }
    }
}
