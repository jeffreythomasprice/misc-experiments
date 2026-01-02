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
        private readonly App app;

        public State(App app)
        {
            this.app = app;
        }

        public void Exit()
        {
            app.log.LogDebug("exit");
            app.window.Close();
        }

        // TODO props here
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
    private readonly BufferWrapper<Vertex2DRgba> vertexBuffer;

    // vulkan stuff that gets recreated periodically, e.g. when display resizes
    private SwapchainWrapper? swapchain;
    private RenderPassWrapper? renderPass;
    private GraphicsPipelineWrapper<Vertex2DRgba>? graphicsPipeline;
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
        vertexBuffer = new BufferWrapper<Vertex2DRgba>(
            vk,
            physicalDevice,
            device, // vulkan stuff that gets recreated periodically, e.g. when display resizes
            [
                new(new Vector2D<float>(0.0f, -0.5f), new Vector4D<float>(1.0f, 0.0f, 0.0f, 1.0f)),
                new(new Vector2D<float>(0.5f, 0.5f), new Vector4D<float>(0.0f, 1.0f, 0.0f, 1.0f)),
                new(new Vector2D<float>(-0.5f, 0.5f), new Vector4D<float>(0.0f, 0.0f, 1.0f, 1.0f)),
            ],
            BufferUsageFlags.VertexBufferBit
        );

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

        if (graphicsPipeline is not null)
        {
            synchronizedQueueSubmitterAndPresenter?.OnRender(
                (commandBuffer) =>
                {
                    // TODO defer to event handler
                    vk.CmdBindPipeline(
                        commandBuffer,
                        PipelineBindPoint.Graphics,
                        graphicsPipeline.GraphicsPipeline
                    );
                    // TODO helper method to automate offsets and draw?
                    var vertexBuffers = new Silk.NET.Vulkan.Buffer[] { vertexBuffer.Buffer };
                    var offsets = new ulong[] { 0 };
                    fixed (ulong* offsetsPtr = offsets)
                    fixed (Silk.NET.Vulkan.Buffer* vertexBuffersPtr = vertexBuffers)
                    {
                        vk.CmdBindVertexBuffers(commandBuffer, 0, 1, vertexBuffersPtr, offsetsPtr);
                    }

                    vk.CmdDraw(commandBuffer, (uint)vertexBuffer.Count, 1, 0, 0);
                },
                out needsRecreate
            );
        }

        // TODO fix event handler stuff to make it possible to make new command queues easily?
        // eventHandler.OnRender(new(this), TimeSpan.FromSeconds(deltaTime));
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

        eventHandler.OnUnload(new(this));

        CleanupStuffThatGetsRecreatedAllTheTime();

        vertexBuffer.Dispose();
        device.Dispose();
        surface.Dispose();
        debugMessenger.Dispose();
        instance.Dispose();
        vk.Dispose();
    }

    private void CleanupStuffThatGetsRecreatedAllTheTime()
    {
        synchronizedQueueSubmitterAndPresenter?.Dispose();
        commandPool?.Dispose();
        graphicsPipeline?.Dispose();
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

        if (swapchainCreatedEventInvokedAtLeastOnce)
        {
            eventHandler.OnSwapchainDestroyed(new(this));
        }
        CleanupStuffThatGetsRecreatedAllTheTime();

        // TODO which parts of this should be owned by the event handler impl?
        swapchain = new SwapchainWrapper(window, vk, instance, surface, physicalDevice, device);
        renderPass = new RenderPassWrapper(vk, device, swapchain);
        graphicsPipeline = new GraphicsPipelineWrapper<Vertex2DRgba>(
            vk,
            device,
            swapchain,
            renderPass,
            File.ReadAllBytes("Shaders/shader.vert.spv"),
            File.ReadAllBytes("Shaders/shader.frag.spv")
        );
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
