using System.Reflection;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.OpenGL;
using Silk.NET.Windowing;

interface IWindowState
{
    Vector2D<int> Size { get; }
}

class WindowState : IWindowState
{
    private readonly IWindow window;

    public WindowState(IWindow window)
    {
        this.window = window;
    }

    public Vector2D<int> Size => window.Size;
}

class AppStateTransition
{
    public static AppStateTransition Exit => new((gl, windowState) => Task.FromResult<IAppState?>(null));

    private readonly Func<GL, IWindowState, Task<IAppState?>> factory;

    public AppStateTransition(Func<GL, IWindowState, IAppState?> factory)
    {
        this.factory = (gl, windowState) => Task.FromResult<IAppState?>(factory(gl, windowState));
    }

    public AppStateTransition(Func<GL, IWindowState, Task<IAppState?>> factory)
    {
        this.factory = factory;
    }

    public async Task<IAppState?> Get(GL gl, IWindowState windowState)
    {
        return await factory(gl, windowState);
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
    private AppStateTransition? initialState;
    private readonly IWindow window;
    private readonly WindowState windowState;
    private GL? openGLContext;

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

    public App(AppStateTransition initialState)
    {
        stateTransitions = new();
        this.initialState = initialState;
        var windowOptions = WindowOptions.Default;
        windowOptions.Size = new(1024, 768);
        windowOptions.Title = "Experiment";
        window = Window.Create(windowOptions);
        window.Load += Load;
        window.Closing += Closing;
        window.FramebufferResize += Resize;
        window.Update += Update;
        window.Render += Render;
        windowState = new WindowState(window);
        window.Run();
    }

    public void Dispose()
    {
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

        openGLContext = GL.GetApi(window);
        if (initialState == null)
        {
            throw new Exception("handling load event but null initial state, did we load twice?");
        }
        HandleTransition(initialState);
        initialState = null;
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

    private GL OpenGLContext
    {
        get
        {
            if (openGLContext == null)
            {
                throw new NullReferenceException("OpenGL context");
            }
            return openGLContext;
        }
    }

    private void HandleTransition(AppStateTransition? transition)
    {
        if (transition != null)
        {
            stateTransitions.Enqueue(transition.Get(OpenGLContext, windowState));
        }
    }
}
