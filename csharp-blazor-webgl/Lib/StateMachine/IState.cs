using BlazorExperiments.Lib.WebGl;
using System.Drawing;

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

    public virtual Task<IState> ResizeAsync(WebGL2RenderingContext gl, Size size)
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

    public virtual Task MouseDown(MouseEvent e)
    {
        return Task.CompletedTask;
    }

    public virtual Task MouseUp(MouseEvent e)
    {
        return Task.CompletedTask;
    }

    public virtual Task MouseMove(MouseMoveEvent e)
    {
        return Task.CompletedTask;
    }

    public virtual Task KeyDown(KeyEvent e)
    {
        return Task.CompletedTask;
    }

    public virtual Task KeyUp(KeyEvent e)
    {
        return Task.CompletedTask;
    }
}
