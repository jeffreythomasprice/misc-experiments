using BlazorExperiments.Lib.WebGl;

namespace BlazorExperiments.Lib.StateMachine;

public class StateMachine : IAsyncDisposable
{
    private readonly WebGL2RenderingContext gl;
    private IState? currentState;
    private TimeSpan? lastAnim;

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

    public async Task ResizeAsync(int width, int height)
    {
        if (currentState != null)
        {
            await PossibleSwitchTo(await currentState.ResizeAsync(gl, width, height));
        }
    }

    public async Task Anim(TimeSpan timeSpan)
    {
        if (currentState != null)
        {
            if (lastAnim != null)
            {
                var delta = lastAnim.Value - timeSpan;
                await PossibleSwitchTo(await currentState.UpdateAsync(gl, delta));
            }
            lastAnim = timeSpan;

            await currentState.RenderAsync(gl);
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
        }
    }
}
