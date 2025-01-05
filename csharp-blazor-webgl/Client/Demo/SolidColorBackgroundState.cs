using BlazorExperiments.Lib.StateMachine;
using BlazorExperiments.Lib.WebGl;

namespace BlazorExperiments.Client.Demo;

public class SolidColorBackgroundState : IState
{
    private readonly double red;
    private readonly double green;
    private readonly double blue;
    private readonly double alpha;

    // TODO use color struct
    public SolidColorBackgroundState(double red, double green, double blue, double alpha)
    {
        this.red = red;
        this.green = green;
        this.blue = blue;
        this.alpha = alpha;
    }

    public override Task RenderAsync(WebGL2RenderingContext gl)
    {
        gl.ClearColor(red, green, blue, alpha);
        gl.Clear(WebGL2RenderingContext.ClearBuffer.COLOR_BUFFER_BIT);
        return Task.CompletedTask;
    }
}
