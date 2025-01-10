using BlazorExperiments.Lib.WebGl;
using System.Drawing;

namespace BlazorExperiments.Lib.StateMachine;

public abstract class IState
{
    public virtual Task ActivateAsync(StateMachine sm, WebGL2RenderingContext gl)
    {
        return Task.CompletedTask;
    }

    public virtual Task DeactivateAsync(StateMachine sm, WebGL2RenderingContext gl)
    {
        return Task.CompletedTask;
    }

    public virtual Task<IState> ResizeAsync(StateMachine sm, WebGL2RenderingContext gl, Size size)
    {
        return Task.FromResult(this);
    }

    public virtual Task<IState> UpdateAsync(StateMachine sm, WebGL2RenderingContext gl, TimeSpan timeSpan)
    {
        return Task.FromResult(this);
    }

    public virtual Task RenderAsync(StateMachine sm, WebGL2RenderingContext gl)
    {
        return Task.CompletedTask;
    }

    public virtual Task MouseDown(StateMachine sm, MouseEvent e)
    {
        return Task.CompletedTask;
    }

    public virtual Task MouseUp(StateMachine sm, MouseEvent e)
    {
        return Task.CompletedTask;
    }

    public virtual Task MouseMove(StateMachine sm, MouseMoveEvent e)
    {
        return Task.CompletedTask;
    }

    public virtual Task KeyDown(StateMachine sm, KeyEvent e)
    {
        return Task.CompletedTask;
    }

    public virtual Task KeyUp(StateMachine sm, KeyEvent e)
    {
        return Task.CompletedTask;
    }
}
