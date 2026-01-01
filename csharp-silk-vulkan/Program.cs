using Experiment;
using Microsoft.Extensions.Logging;

var logger = LoggerUtils.Factory.Value.CreateLogger<Program>();
logger.LogInformation("start");

using var app = new App(
    new App.CreateOptions
    {
        Title = "Experiment",
        Size = new(1280, 720),
        FixedSize = false,
    },
    new EventHandler()
);
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
