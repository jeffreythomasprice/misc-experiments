using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.WebGPU;
using Silk.NET.Windowing;
using System.Reflection;
using System.Runtime.InteropServices;

interface IWindowState
{
    Vector2D<int> Size { get; }
    WebGPU WebGPU { get; }
    unsafe Surface* Surface { get; }
    unsafe Device* Device { get; }
}

unsafe class WindowState : IWindowState, IDisposable
{
    private readonly IWindow window;
    private readonly WebGPU webGPU;
    private readonly Instance* instance;
    private readonly Surface* surface;

    private readonly Adapter* adapter;
    private readonly Device* device;

    public WindowState(IWindow window)
    {
        this.window = window;

        webGPU = CreateWebGPU();
        instance = CreateInstance(webGPU);
        surface = CreateSurface(window, webGPU, instance);
        adapter = CreateAdapter(webGPU, instance, surface);
        device = CreateDevice(webGPU, adapter);
        ConfigureSurface(window, webGPU, surface, device);
        ConfigureDebugCallback(webGPU, device);
    }

    public Vector2D<int> Size => window.Size;

    public WebGPU WebGPU => webGPU;

    public unsafe Surface* Surface => surface;

    public unsafe Device* Device => device;

    public void Dispose()
    {
        webGPU.DeviceDestroy(device);
        webGPU.SurfaceRelease(surface);
        webGPU.AdapterRelease(adapter);
        webGPU.InstanceRelease(instance);
        Console.WriteLine("webGPU resources released");
    }

    private static WebGPU CreateWebGPU()
    {
        return WebGPU.GetApi();
    }

    private static Instance* CreateInstance(WebGPU webGPU)
    {
        var descriptor = new InstanceDescriptor();
        var result = webGPU.CreateInstance(ref descriptor);
        Console.WriteLine("created instance");
        return result;
    }

    private static Surface* CreateSurface(IWindow window, WebGPU webGPU, Instance* instance)
    {
        var result = window.CreateWebGPUSurface(webGPU, instance);
        Console.WriteLine("created surface");
        return result;
    }

    private static Adapter* CreateAdapter(WebGPU webGPU, Instance* instance, Surface* surface)
    {
        Adapter* result = null;
        Exception? error = null;

        var options = new RequestAdapterOptions
        {
            CompatibleSurface = surface,
            BackendType = BackendType.Vulkan,
            PowerPreference = PowerPreference.HighPerformance
        };
        var callback = PfnRequestAdapterCallback.From((status, adapter, msgPtr, userDataPtr) =>
        {
            if (status == RequestAdapterStatus.Success)
            {
                result = adapter;
            }
            else
            {
                error = new Exception($"error getting adapter: {Marshal.PtrToStringAnsi((IntPtr)msgPtr)}");
            }
        });
        webGPU.InstanceRequestAdapter(instance, ref options, callback, null);

        if (error != null)
        {
            throw error;
        }
        if (result == null)
        {
            throw new Exception($"didn't create adapter, completed without callback being invoked");
        }

        var adapterProperties = new AdapterProperties();
        webGPU.AdapterGetProperties(result, ref adapterProperties);
        Console.WriteLine($"adapter type: {adapterProperties.AdapterType}");
        Console.WriteLine($"adapter architecture: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.Architecture)}");
        Console.WriteLine($"adapter backend type: {adapterProperties.BackendType}");
        Console.WriteLine($"adapter device ID: {adapterProperties.DeviceID}");
        Console.WriteLine($"adapter driver description: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.DriverDescription)}");
        Console.WriteLine($"adapter name: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.Name)}");
        Console.WriteLine($"adapter vendor ID: {adapterProperties.VendorID}");
        Console.WriteLine($"adapter vendor name: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.VendorName)}");

        return result;
    }

    private static Device* CreateDevice(WebGPU webGPU, Adapter* adapter)
    {
        Device* result = null;
        Exception? error = null;

        var descriptor = new DeviceDescriptor();
        var callback = PfnRequestDeviceCallback.From((status, device, msgPtr, userDataPtr) =>
        {
            if (status == RequestDeviceStatus.Success)
            {
                result = device;
            }
            else
            {
                error = new Exception($"error getting device: {Marshal.PtrToStringAnsi((IntPtr)msgPtr)}");
            }
        });
        webGPU.AdapterRequestDevice(adapter, ref descriptor, callback, null);

        if (error != null)
        {
            throw error;
        }
        if (result == null)
        {
            throw new Exception($"didn't create adapter, completed without callback being invoked");
        }

        Console.WriteLine("created device");

        return result;
    }

    private static void ConfigureSurface(IWindow window, WebGPU webGPU, Surface* surface, Device* device)
    {
        var configuration = new SurfaceConfiguration()
        {
            Device = device,
            Width = (uint)window.Size.X,
            Height = (uint)window.Size.Y,
            Format = TextureFormat.Bgra8Unorm,
            PresentMode = PresentMode.Fifo,
            Usage = TextureUsage.RenderAttachment,
        };
        webGPU.SurfaceConfigure(surface, ref configuration);
    }

    private static void ConfigureDebugCallback(WebGPU webGPU, Device* device)
    {
        var callback = PfnErrorCallback.From((type, msgPtr, userDataPtr) =>
        {
            Console.WriteLine($"unhandled WebGPU error: {Marshal.PtrToStringAnsi((IntPtr)msgPtr)}");
        });
        webGPU.DeviceSetUncapturedErrorCallback(device, callback, null);
    }
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

    public App(AppStateTransition initialState)
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

        windowState = new WindowState(window);

        HandleTransition(initialState);

        window.Run();
    }

    public void Dispose()
    {
        windowState.Dispose();
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
