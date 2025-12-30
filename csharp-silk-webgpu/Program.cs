using Experiment;
using Silk.NET.WebGPU;

using var app = new App(new EventHandler());
app.Run();

unsafe class EventHandler : IAppEventHandler
{
    private Pipeline? pipeline;

    public void OnLoad(App.State state)
    {
        pipeline = new Pipeline(state);
    }

    public void OnUnload(App.State state)
    {
        pipeline?.Dispose();
        pipeline = null;
    }

    public void OnRender(App.State state, TimeSpan deltaTime, RenderPassEncoder* renderPassEncoder)
    {
        pipeline?.Render(renderPassEncoder);
    }
}
