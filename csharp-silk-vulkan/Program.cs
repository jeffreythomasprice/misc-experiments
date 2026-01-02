using Experiment;
using Microsoft.Extensions.Logging;
using Silk.NET.Input;

var log = LoggerUtils.Factory.Value.CreateLogger<Program>();
log.LogInformation("start");

using var app = new App(
    new App.CreateOptions
    {
        Title = "Experiment",
        Size = new(1280, 720),
        FixedSize = false,
    },
    new Demo()
);
app.Run();

class Demo : IAppEventHandler
{
    private readonly ILogger<Demo> log;

    public Demo()
    {
        log = LoggerUtils.Factory.Value.CreateLogger<Demo>();
    }

    public void OnLoad(App.State state)
    {
        log.LogDebug("TODO Demo OnLoad");
    }

    public void OnSwapchainCreated(App.State state)
    {
        log.LogDebug("TODO Demo OnSwapchainCreated");
    }

    public void OnSwapchainDestroyed(App.State state)
    {
        log.LogDebug("TODO Demo OnSwapchainDestroyed");
    }

    public void OnUnload(App.State state)
    {
        log.LogDebug("TODO Demo OnUnload");
    }

    public void OnRender(App.State state, TimeSpan deltaTime)
    {
        // TODO render
        log.LogDebug("TODO Demo OnRender dt={DeltaTime}", deltaTime);
    }

    public void OnKeyUp(App.State state, IKeyboard keyboard, Key key, int keyCode)
    {
        if (key == Key.Escape)
        {
            state.Exit();
        }
    }
}
