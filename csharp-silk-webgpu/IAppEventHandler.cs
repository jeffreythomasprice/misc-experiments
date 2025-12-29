using Silk.NET.WebGPU;

namespace Experiment;

public interface IAppEventHandler
{
    public void OnLoad(App.State state) { }
    public void OnUnload(App.State state) { }
    public unsafe void OnRender(
        App.State state,
        TimeSpan deltaTime,
        RenderPassEncoder* renderPassEncoder
    ) { }
}
