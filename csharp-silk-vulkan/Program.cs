using Experiment;

using var app = new App(new EventHandler());
app.Run();

unsafe class EventHandler : IAppEventHandler
{
    public void OnLoad(App.State state)
    {
        // TODO init
    }

    public void OnUnload(App.State state)
    {
        // TODO cleanup
    }

    public void OnRender(App.State state, TimeSpan deltaTime)
    {
        // TODO render
    }
}
