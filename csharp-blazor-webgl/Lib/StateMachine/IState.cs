using BlazorExperiments.Lib.WebGl;

namespace BlazorExperiments.Lib.StateMachine;

public abstract class IState
{
    public virtual Task ActivateAsync(WebGL2RenderingContext gl)
    {
        return Task.CompletedTask;
    }

    public virtual Task DeactivateAsync(WebGL2RenderingContext gl)
    {
        return Task.CompletedTask;
    }

    public virtual Task<IState> ResizeAsync(WebGL2RenderingContext gl, int width, int height)
    {
        return Task.FromResult(this);
    }

    public virtual Task<IState> UpdateAsync(WebGL2RenderingContext gl, TimeSpan timeSpan)
    {
        return Task.FromResult(this);
    }

    public virtual Task RenderAsync(WebGL2RenderingContext gl)
    {
        return Task.CompletedTask;

    }
}
