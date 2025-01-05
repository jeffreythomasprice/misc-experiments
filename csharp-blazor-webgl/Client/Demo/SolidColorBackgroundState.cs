using BlazorExperiments.Lib.Math;
using BlazorExperiments.Lib.StateMachine;
using BlazorExperiments.Lib.WebGl;

namespace BlazorExperiments.Client.Demo;

public class SolidColorBackgroundState : IState
{
    private readonly ColorRGBA<double> color;

    public SolidColorBackgroundState(ColorRGBA<double> color)
    {
        this.color = color;
    }

    public override Task RenderAsync(WebGL2RenderingContext gl)
    {
        gl.ClearColor(color);
        gl.Clear(WebGL2RenderingContext.ClearBuffer.COLOR_BUFFER_BIT);
        return Task.CompletedTask;
    }
}
