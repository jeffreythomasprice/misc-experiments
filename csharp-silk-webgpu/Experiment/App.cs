using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.Windowing;
using System.Reflection;

interface IVideoDriver : IDisposable { }

interface IWindowState
{
    Vector2D<int> Size { get; }
    IVideoDriver VideoDriver { get; }
}

unsafe class WindowState : IWindowState
{
    private readonly IWindow window;
    private readonly IVideoDriver videoDriver;

    public WindowState(IWindow window, IVideoDriver videoDriver)
    {
        this.window = window;
        this.videoDriver = videoDriver;
    }

    public Vector2D<int> Size => window.Size;

    public IVideoDriver VideoDriver => videoDriver;
}

class AppStateTransition
{
    public static AppStateTransition Exit => new((windowState) => Task.FromResult<IAppState?>(null));

    private readonly Func<IWindowState, Task<IAppState?>> factory;

    public AppStateTransition(Func<IWindowState, IAppState?> factory)
    {
        this.factory = (windowState) => Task.FromResult<IAppState?>(factory(windowState));
    }

    public AppStateTransition(Func<IWindowState, Task<IAppState?>> factory)
    {
        this.factory = factory;
    }

    public async Task<IAppState?> Get(IWindowState windowState)
    {
        return await factory(windowState);
    }
}

interface IAppState
{
    void Load();
    void Unload();
    void Resize(Vector2D<int> size);
    AppStateTransition? KeyDown(Key key);
    AppStateTransition? KeyUp(Key key);
    AppStateTransition? Update(TimeSpan delta);
    void Render();
}

class App : IDisposable
{
    private readonly Queue<Task<IAppState?>> stateTransitions;
    private readonly IWindow window;
    private readonly IVideoDriver videoDriver;
    private readonly WindowState windowState;

    private IAppState? state;

    public static Stream EmbeddedFileAsStream(string name)
    {
        return Assembly.GetExecutingAssembly().GetManifestResourceStream(name)
            ?? throw new Exception($"failed to find embedded file: {name}");
    }

    public static string EmbeddedFileAsString(string name)
    {
        using var stream = EmbeddedFileAsStream(name);
        using var reader = new StreamReader(stream);
        return reader.ReadToEnd();
    }

    public App(Func<IWindow, IVideoDriver> videoDriverFactory, AppStateTransition initialState)
    {
        stateTransitions = new();

        var windowOptions = WindowOptions.Default;
        windowOptions.Size = new(1024, 768);
        windowOptions.Title = "Experiment";
        windowOptions.API = GraphicsAPI.None;

        window = Window.Create(windowOptions);

        window.Load += Load;
        window.Closing += Closing;
        window.FramebufferResize += Resize;
        window.Update += Update;
        window.Render += Render;

        window.Initialize();

        videoDriver = videoDriverFactory(window);
        windowState = new WindowState(window, videoDriver);

        HandleTransition(initialState);

        window.Run();
    }

    public void Dispose()
    {
        videoDriver.Dispose();
        window.Dispose();
    }

    private void Load()
    {
        var input = window.CreateInput();
        foreach (var keyboard in input.Keyboards)
        {
            keyboard.KeyDown += KeyDown;
            keyboard.KeyUp += KeyUp;
        }
    }

    private void Closing()
    {
        // we won't have time to do any state transitions, so just try to clean up the active state
        state?.Unload();
        state = null;
    }

    private void Resize(Vector2D<int> size)
    {
        state?.Resize(size);
    }

    private void Update(double time)
    {
        if (stateTransitions.TryDequeue(out var nextStateTask) && nextStateTask != null && nextStateTask.IsCompleted)
        {
            var nextState = nextStateTask.Result;
            if (nextState == null)
            {
                state?.Unload();
                state = null;
                window.Close();
            }
            else
            {
                state?.Unload();
                nextState.Load();
                state = nextState;
            }
        }
        else
        {
            HandleTransition(state?.Update(TimeSpan.FromSeconds(time)));
        }
    }

    private void Render(double time)
    {
        state?.Render();
    }

    private void KeyDown(IKeyboard keyboard, Key key, int unknown)
    {
        HandleTransition(state?.KeyDown(key));
    }

    private void KeyUp(IKeyboard keyboard, Key key, int unknown)
    {
        HandleTransition(state?.KeyUp(key));
    }

    private void HandleTransition(AppStateTransition? transition)
    {
        if (transition != null)
        {
            stateTransitions.Enqueue(transition.Get(windowState));
        }
    }
}
