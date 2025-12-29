using System;
using System.Runtime.InteropServices;
using Silk.NET.Maths;
using Silk.NET.WebGPU;
using Silk.NET.Windowing;

// TODO do logging correctly

// TODO handle resize events

namespace Experiment;

public sealed unsafe class App : IDisposable
{
    public class State
    {
        private readonly App app;

        public State(App app)
        {
            this.app = app;
        }

        public WebGPU WGPU => app.wgpu;
        public Device* Device => app.device;
        public TextureFormat PreferredTextureFormat => app.preferredTextureFormat;
    }

    private readonly IAppEventHandler eventHandler;

    private readonly IWindow window;
    private WebGPU wgpu;
    private Instance* instance;
    private Surface* surface;
    private Adapter* adapter;
    private Device* device;
    private readonly TextureFormat preferredTextureFormat;

    private bool isCleanupDone = false;

    public App(IAppEventHandler eventHandler)
    {
        this.eventHandler = eventHandler;

        var windowOptions = WindowOptions.Default;
        windowOptions.Size = new Vector2D<int>(1280, 720);
        windowOptions.Title = "Experiment";
        windowOptions.API = GraphicsAPI.None;
        window = Window.Create(windowOptions);

        /*
        don't rely on the OnLoad callback, we have to call Initialize manually before we can init WebGPU stuff, and we need that to call the
        event handler's OnLoad
        */
        window.Render += OnRender;
        window.Closing += OnClosing;

        window.Initialize();

        wgpu = CreateAPI();
        instance = CreateInstance(wgpu);
        surface = CreateSurface(window, wgpu, instance);
        adapter = CreateAdapter(wgpu, instance, surface);
        device = CreateDevice(wgpu, adapter);
        preferredTextureFormat = ConfigureSurface(window, wgpu, surface, device);
        ConfigureDebugCallback(wgpu, device);

        eventHandler.OnLoad(new(this));
    }

    public void Dispose()
    {
        window?.Dispose();
        Cleanup();
        GC.SuppressFinalize(this);
    }

    public void Run()
    {
        window.Run();
    }

    private void OnRender(double deltaTime)
    {
        var queue = wgpu.DeviceGetQueue(device);
        var currentCommandEncoder = wgpu.DeviceCreateCommandEncoder(device, null);

        SurfaceTexture surfaceTexture;
        wgpu.SurfaceGetCurrentTexture(surface, &surfaceTexture);
        var surfaceTextureView = wgpu.TextureCreateView(surfaceTexture.Texture, null);

        var colorAttachments = stackalloc RenderPassColorAttachment[1];
        colorAttachments[0].View = surfaceTextureView;
        colorAttachments[0].LoadOp = LoadOp.Clear;
        colorAttachments[0].ClearValue = new Color(0.1f, 0.2f, 0.3f, 1.0f);
        colorAttachments[0].StoreOp = StoreOp.Store;

        var renderPassDescriptor = new RenderPassDescriptor();
        renderPassDescriptor.ColorAttachments = colorAttachments;
        renderPassDescriptor.ColorAttachmentCount = 1;

        var currentRenderPassEncoder = wgpu.CommandEncoderBeginRenderPass(
            currentCommandEncoder,
            &renderPassDescriptor
        );

        eventHandler.OnRender(new(this), TimeSpan.FromSeconds(deltaTime), currentRenderPassEncoder);

        wgpu.RenderPassEncoderEnd(currentRenderPassEncoder);

        var commandBuffer = wgpu.CommandEncoderFinish(currentCommandEncoder, null);

        wgpu.QueueSubmit(queue, 1, &commandBuffer);
        wgpu.SurfacePresent(surface);

        wgpu.TextureViewRelease(surfaceTextureView);
        wgpu.TextureRelease(surfaceTexture.Texture);
        wgpu.RenderPassEncoderRelease(currentRenderPassEncoder);
        wgpu.CommandBufferRelease(commandBuffer);
        wgpu.CommandEncoderRelease(currentCommandEncoder);
    }

    private void OnClosing()
    {
        Console.WriteLine("Window closing");
        Cleanup();
    }

    private void Cleanup()
    {
        if (isCleanupDone)
        {
            return;
        }
        isCleanupDone = true;

        eventHandler.OnUnload(new(this));

        if (device != null)
        {
            wgpu.DeviceDestroy(device);
            Console.WriteLine("WGPU Device destroyed");
            device = null;
        }

        if (surface != null)
        {
            wgpu.SurfaceRelease(surface);
            Console.WriteLine("WGPU Surface released");
            surface = null;
        }

        if (adapter != null)
        {
            wgpu.AdapterRelease(adapter);
            Console.WriteLine("WGPU Adapter released");
            adapter = null;
        }

        if (instance != null)
        {
            wgpu.InstanceRelease(instance);
            Console.WriteLine("WGPU Instance released");
            instance = null;
        }
    }

    private static WebGPU CreateAPI()
    {
        var result = WebGPU.GetApi();
        return result;
    }

    private static Instance* CreateInstance(WebGPU wgpu)
    {
        var instanceDescriptor = new InstanceDescriptor();
        var result = wgpu.CreateInstance(&instanceDescriptor);
        Console.WriteLine("WGPU Instance created");
        return result;
    }

    private static Surface* CreateSurface(IWindow window, WebGPU wgpu, Instance* instance)
    {
        var result = window.CreateWebGPUSurface(wgpu, instance);
        Console.WriteLine("WGPU Surface created");
        return result;
    }

    private static Adapter* CreateAdapter(WebGPU wgpu, Instance* instance, Surface* surface)
    {
        Adapter* result = null;

        var adapterCallback = PfnRequestAdapterCallback.From(
            (status, wgpuAdapter, msgPtr, userDataPtr) =>
            {
                if (status == RequestAdapterStatus.Success)
                {
                    result = wgpuAdapter;
                    Console.WriteLine("Retrieved WGPU Adapter");
                }
                else
                {
                    string msg = Marshal.PtrToStringAnsi((IntPtr)msgPtr) ?? "Unknown error";
                    Console.WriteLine($"Error while retrieving WGPU Adapter: {msg}");
                }
            }
        );

        var options = new RequestAdapterOptions();
        options.CompatibleSurface = surface;
        options.BackendType = BackendType.Vulkan;
        options.PowerPreference = PowerPreference.HighPerformance;

        wgpu.InstanceRequestAdapter(instance, &options, adapterCallback, null);

        // TODO block here until result is set
        if (result == null)
        {
            throw new NullReferenceException("Failed to obtain WGPU Adapter.");
        }

        return result;
    }

    private static Device* CreateDevice(WebGPU wgpu, Adapter* adapter)
    {
        Device* result = null;

        var deviceCallback = PfnRequestDeviceCallback.From(
            (status, wgpuDevice, msgPtr, userDataPtr) =>
            {
                if (status == RequestDeviceStatus.Success)
                {
                    result = wgpuDevice;
                    Console.WriteLine("Retrieved WGPU Device");
                }
                else
                {
                    string msg = Marshal.PtrToStringAnsi((IntPtr)msgPtr) ?? "Unknown error";
                    Console.WriteLine($"Error while retrieving WGPU Device: {msg}");
                }
            }
        );

        var descriptor = new DeviceDescriptor();

        wgpu.AdapterRequestDevice(adapter, &descriptor, deviceCallback, null);

        // TODO block here until result is set
        if (result == null)
        {
            throw new NullReferenceException("Failed to obtain WGPU Device.");
        }

        return result;
    }

    private static TextureFormat ConfigureSurface(
        IWindow window,
        WebGPU wgpu,
        Surface* surface,
        Device* device
    )
    {
        var preferredTextureFormat = TextureFormat.Bgra8Unorm;
        var configuration = new SurfaceConfiguration
        {
            Device = device,
            Width = (uint)window.Size.X,
            Height = (uint)window.Size.Y,
            Format = preferredTextureFormat,
            PresentMode = PresentMode.Immediate,
            Usage = TextureUsage.RenderAttachment,
        };
        wgpu.SurfaceConfigure(surface, &configuration);

        Console.WriteLine("WGPU Surface configured");

        return preferredTextureFormat;
    }

    private static void ConfigureDebugCallback(WebGPU wgpu, Device* device)
    {
        var errorCallback = PfnErrorCallback.From(
            (type, msgPtr, userDataPtr) =>
            {
                string msg = Marshal.PtrToStringAnsi((IntPtr)msgPtr) ?? "Unknown error";
                Console.WriteLine($"WGPU Error: {msg}");
            }
        );
        wgpu.DeviceSetUncapturedErrorCallback(device, errorCallback, null);
    }
}
