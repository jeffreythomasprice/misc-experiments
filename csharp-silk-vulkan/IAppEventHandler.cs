namespace Experiment;

using Experiment.VulkanUtils;
using Silk.NET.Input;

public interface IAppEventHandler
{
    public void OnLoad(App.State state) { }
    public void OnSwapchainCreated(App.GraphicsReadyState state) { }
    public void OnSwapchainDestroyed(App.GraphicsReadyState state) { }
    public void OnUnload(App.State state) { }
    public void OnRender(
        App.GraphicsReadyState state,
        CommandBufferWrapper commandBuffer,
        TimeSpan deltaTime
    ) { }
    public void OnUpdate(App.State state, TimeSpan deltaTime) { }
    public void OnResize(App.State state) { }
    public void OnKeyDown(App.State state, IKeyboard keyboard, Key key, int keyCode) { }
    public void OnKeyUp(App.State state, IKeyboard keyboard, Key key, int keyCode) { }
}
