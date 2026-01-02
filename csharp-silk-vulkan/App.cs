namespace Experiment;

using System;
using Experiment.VulkanUtils;
using Microsoft.Extensions.Logging;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.Vulkan;
using Silk.NET.Windowing;

// TODO next tutorial
// https://github.com/dfkeenan/SilkVulkanTutorial/blob/main/Source/20_StagingBuffer/Program.cs

public sealed unsafe partial class App : IDisposable
{
    public record struct CreateOptions
    {
        public string Title;
        public Vector2D<int> Size;
        public bool FixedSize;
    }

    public class State
    {
        protected readonly App app;

        public State(App app)
        {
            this.app = app;
        }

        public void Exit()
        {
            app.log.LogDebug("exit");
            app.window.Close();
        }

        public Vk Vk => app.vk;

        public PhysicalDeviceWrapper PhysicalDevice => app.physicalDevice;

        public DeviceWrapper Device => app.device;
    }

    public class GraphicsReadyState : State
    {
        public GraphicsReadyState(App app)
            : base(app) { }

        public SwapchainWrapper Swapchain =>
            app.swapchain ?? throw new InvalidOperationException("not initialized yet");

        public RenderPassWrapper RenderPass =>
            app.renderPass ?? throw new InvalidOperationException("not initialized yet");

        public CommandPoolWrapper CommandPool =>
            app.commandPool ?? throw new InvalidOperationException("not initialized yet");
    }

    private readonly ILogger log;
    private readonly IAppEventHandler eventHandler;

    private readonly IWindow window;

    // vulkan stuff that stays alive forever
    private readonly Vk vk;
    private readonly InstanceWrapper instance;
    private readonly DebugMessengerWrapper debugMessenger;
    private readonly SurfaceWrapper surface;
    private readonly PhysicalDeviceWrapper physicalDevice;
    private readonly DeviceWrapper device;

    // vulkan stuff that gets recreated periodically, e.g. when display resizes
    private SwapchainWrapper? swapchain;
    private RenderPassWrapper? renderPass;
    private CommandPoolWrapper? commandPool;
    private SynchronizedQueueSubmitterAndPresenter? synchronizedQueueSubmitterAndPresenter;

    private bool swapchainCreatedEventInvokedAtLeastOnce = false;

    private bool isCleanupDone = false;

    // flag that indicates we might need to recreate stuff next time
    private bool needsRecreate = true;

    public App(CreateOptions createOptions, IAppEventHandler eventHandler)
    {
        log = LoggerUtils.Factory.Value.CreateLogger(GetType());
        this.eventHandler = eventHandler;

        var windowOptions = WindowOptions.DefaultVulkan with
        {
            Title = createOptions.Title,
            Size = createOptions.Size,
        };
        if (createOptions.FixedSize)
        {
            windowOptions.WindowBorder = WindowBorder.Fixed;
        }
        window = Window.Create(windowOptions);
        if (createOptions.FixedSize)
        {
            window.Center(Monitor.GetMainMonitor(null));
        }

        window.Load += OnLoad;
        window.Render += OnRender;
        window.Update += OnUpdate;
        window.Closing += OnClosing;
        window.Resize += OnResize;

        window.Initialize();

        if (window.VkSurface is null)
        {
            throw new Exception("Missing Vulkan surface");
        }

        var enableValidationLayers = true;
        vk = Vk.GetApi();
        instance = new InstanceWrapper(vk, window.VkSurface, enableValidationLayers);
        debugMessenger = new DebugMessengerWrapper(vk, instance, enableValidationLayers);
        surface = new SurfaceWrapper(window.VkSurface, vk, instance);
        physicalDevice = PhysicalDeviceWrapper.FindBest(vk, instance.Instance, surface);
        device = new DeviceWrapper(vk, physicalDevice, enableValidationLayers);

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

    private void OnLoad()
    {
        log.LogTrace("window load");

        var input = window.CreateInput();

        foreach (var keyboard in input.Keyboards)
        {
            keyboard.KeyDown += OnKeyDown;
            keyboard.KeyUp += OnKeyUp;
        }
    }

    private void OnRender(double deltaTime)
    {
        if (needsRecreate)
        {
            needsRecreate = false;
            RecreateStuffThatGetsRecreatedAllTheTime();
        }

        if (swapchainCreatedEventInvokedAtLeastOnce)
        {
            synchronizedQueueSubmitterAndPresenter?.OnRender(
                (commandBuffer) =>
                {
                    eventHandler.OnRender(
                        new(this),
                        commandBuffer,
                        TimeSpan.FromSeconds(deltaTime)
                    );
                },
                out needsRecreate
            );
        }
    }

    private void OnUpdate(double deltaTime)
    {
        eventHandler.OnUpdate(new(this), TimeSpan.FromSeconds(deltaTime));
    }

    private void OnClosing()
    {
        log.LogTrace("window closing");
        Cleanup();
    }

    private void OnResize(Vector2D<int> size)
    {
        needsRecreate = true;
    }

    private void OnKeyDown(IKeyboard keyboard, Key key, int keyCode)
    {
        eventHandler.OnKeyDown(new(this), keyboard, key, keyCode);
    }

    private void OnKeyUp(IKeyboard keyboard, Key key, int keyCode)
    {
        eventHandler.OnKeyUp(new(this), keyboard, key, keyCode);
    }

    private void Cleanup()
    {
        if (isCleanupDone)
        {
            return;
        }
        isCleanupDone = true;

        CleanupStuffThatGetsRecreatedAllTheTime();

        eventHandler.OnUnload(new(this));

        device.Dispose();
        surface.Dispose();
        debugMessenger.Dispose();
        instance.Dispose();
        vk.Dispose();
    }

    private void CleanupStuffThatGetsRecreatedAllTheTime()
    {
        if (swapchainCreatedEventInvokedAtLeastOnce)
        {
            eventHandler.OnSwapchainDestroyed(new(this));
        }

        synchronizedQueueSubmitterAndPresenter?.Dispose();
        commandPool?.Dispose();
        renderPass?.Dispose();
        swapchain?.Dispose();
    }

    private void RecreateStuffThatGetsRecreatedAllTheTime()
    {
        var framebufferSize = window.FramebufferSize;
        log.LogTrace(
            "recreating swpachain, renderpass, etc., current frame buffer size {FramebufferSize}, current swapchain extent {SwapchainExtentWidth}x{SwapchainExtentHeight}",
            framebufferSize,
            swapchain?.Extent.Width,
            swapchain?.Extent.Height
        );

        if (
            framebufferSize.X == swapchain?.Extent.Width
            && framebufferSize.Y == swapchain?.Extent.Height
        )
        {
            log.LogTrace("framebuffer size matches swapchain extent, no need to recreate");
            return;
        }

        vk.DeviceWaitIdle(device.Device);

        CleanupStuffThatGetsRecreatedAllTheTime();

        swapchain = new SwapchainWrapper(window, vk, instance, surface, physicalDevice, device);
        renderPass = new RenderPassWrapper(vk, device, swapchain);
        commandPool = new CommandPoolWrapper(vk, physicalDevice, device);
        synchronizedQueueSubmitterAndPresenter = new SynchronizedQueueSubmitterAndPresenter(
            vk,
            device,
            swapchain,
            renderPass,
            commandPool
        );

        eventHandler.OnSwapchainCreated(new(this));
        swapchainCreatedEventInvokedAtLeastOnce = true;
    }
}
