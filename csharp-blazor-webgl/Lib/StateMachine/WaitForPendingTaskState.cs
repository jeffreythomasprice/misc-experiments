using BlazorExperiments.Lib.WebGl;
using System.Drawing;

namespace BlazorExperiments.Lib.StateMachine;

public class WaitForPendingTaskState : IState
{
    private readonly IState interimState;
    private Task<IState>? waitForTask;
    private Func<WebGL2RenderingContext, Task<IState>>? waitForFunc;

    public WaitForPendingTaskState(IState interimState, Task<IState> waitFor)
    {
        this.interimState = interimState;
        this.waitForTask = waitFor;
    }

    public WaitForPendingTaskState(IState interimState, Func<WebGL2RenderingContext, Task<IState>> waitFor)
    {
        this.interimState = interimState;
        this.waitForFunc = waitFor;
    }

    public override async Task ActivateAsync(StateMachine sm, WebGL2RenderingContext gl)
    {
        if (waitForFunc != null && waitForTask == null)
        {
            waitForTask = waitForFunc(gl);
        }

        await interimState.ActivateAsync(sm, gl);
    }

    public override async Task DeactivateAsync(StateMachine sm, WebGL2RenderingContext gl)
    {
        await interimState.DeactivateAsync(sm, gl);
    }

    public override async Task<IState> ResizeAsync(StateMachine sm, WebGL2RenderingContext gl, Size size)
    {
        await interimState.ResizeAsync(sm, gl, size);
        return this;
    }

    public override async Task<IState> UpdateAsync(StateMachine sm, WebGL2RenderingContext gl, TimeSpan timeSpan)
    {
        if (waitForTask?.IsCompleted == true)
        {
            return waitForTask.Result;
        }

        await interimState.UpdateAsync(sm, gl, timeSpan);
        return this;
    }

    public override async Task RenderAsync(StateMachine sm, WebGL2RenderingContext gl)
    {
        await interimState.RenderAsync(sm, gl);
    }

    public override async Task MouseDown(StateMachine sm, MouseEvent e)
    {
        await interimState.MouseDown(sm, e);
    }

    public override async Task MouseUp(StateMachine sm, MouseEvent e)
    {
        await interimState.MouseUp(sm, e);
    }

    public override async Task MouseMove(StateMachine sm, MouseMoveEvent e)
    {
        await interimState.MouseMove(sm, e);
    }

    public override async Task KeyDown(StateMachine sm, KeyEvent e)
    {
        await interimState.KeyDown(sm, e);
    }

    public override async Task KeyUp(StateMachine sm, KeyEvent e)
    {
        await interimState.KeyUp(sm, e);
    }
}
