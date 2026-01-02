namespace Experiment;

using Silk.NET.Input;

public interface IAppEventHandler
{
    public void OnLoad(App.State state) { }
    public void OnSwapchainCreated(App.State state) { }
    public void OnSwapchainDestroyed(App.State state) { }
    public void OnUnload(App.State state) { }
    public void OnRender(App.State state, TimeSpan deltaTime) { }
    public void OnUpdate(App.State state, TimeSpan deltaTime) { }
    public void OnKeyDown(App.State state, IKeyboard keyboard, Key key, int keyCode) { }
    public void OnKeyUp(App.State state, IKeyboard keyboard, Key key, int keyCode) { }
}
