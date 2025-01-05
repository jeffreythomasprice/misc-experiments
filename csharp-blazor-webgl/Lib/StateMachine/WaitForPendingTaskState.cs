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

    public override async Task ActivateAsync(WebGL2RenderingContext gl)
    {
        if (waitForFunc != null && waitForTask == null)
        {
            waitForTask = waitForFunc(gl);
        }

        await interimState.ActivateAsync(gl);
    }

    public override async Task DeactivateAsync(WebGL2RenderingContext gl)
    {
        await interimState.DeactivateAsync(gl);
    }

    public override async Task<IState> ResizeAsync(WebGL2RenderingContext gl, Size size)
    {
        await interimState.ResizeAsync(gl, size);
        return this;
    }

    public override async Task<IState> UpdateAsync(WebGL2RenderingContext gl, TimeSpan timeSpan)
    {
        if (waitForTask?.IsCompleted == true)
        {
            return waitForTask.Result;
        }

        await interimState.UpdateAsync(gl, timeSpan);
        return this;
    }

    public override async Task RenderAsync(WebGL2RenderingContext gl)
    {
        await interimState.RenderAsync(gl);
    }
}
