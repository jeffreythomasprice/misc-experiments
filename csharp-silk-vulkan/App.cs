namespace Experiment;

using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Experiment.VulkanUtils;
using Microsoft.Extensions.Logging;
using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Core.Native;
using Silk.NET.Maths;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.EXT;
using Silk.NET.Vulkan.Extensions.KHR;
using Silk.NET.Windowing;

// TODO handle resize events

// TODO handle keyboard events

// TODO next tutorial
// https://github.com/dfkeenan/SilkVulkanTutorial/blob/main/Source/17_SwapChainRecreation/Program.cs

public sealed unsafe partial class App : IDisposable
{
    public class State
    {
        private readonly App app;

        public State(App app)
        {
            this.app = app;
        }

        // TODO props here
    }

    private readonly ILogger log;
    private readonly IAppEventHandler eventHandler;

    private readonly IWindow window;

    private readonly Vk vk;
    private readonly InstanceWrapper instance;
    private readonly DebugMessengerWrapper debugMessenger;
    private readonly SurfaceWrapper surface;
    private readonly PhysicalDeviceWrapper physicalDevice;
    private readonly DeviceWrapper device;
    private readonly SwapchainWrapper swapchain;
    private readonly RenderPassWrapper renderPass;
    private readonly GraphicsPipelineWrapper graphicsPipeline;
    private readonly CommandPoolWrapper commandPool;
    private readonly SynchronizedQueueSubmitterAndPresenter synchronizedQueueSubmitterAndPresenter;

    private bool isCleanupDone = false;

    public App(IAppEventHandler eventHandler)
    {
        log = LoggerUtils.Factory.Value.CreateLogger(GetType());
        this.eventHandler = eventHandler;

        window = Window.Create(
            WindowOptions.DefaultVulkan with
            {
                Size = new Vector2D<int>(1280, 720),
                Title = "Experiment",
            }
        );

        /*
        don't rely on the OnLoad callback, we have to call Initialize manually before we can init Vulkan stuff, and we need that to call the
        event handler's OnLoad
        */
        // TODO OnUpdate
        window.Render += OnRender;
        window.Closing += OnClosing;
        // TODO Resize or FramebufferResize?
        window.Resize += OnResize;
        window.FramebufferResize += OnResize;

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
        swapchain = new SwapchainWrapper(window, vk, instance, surface, physicalDevice, device);
        renderPass = new RenderPassWrapper(vk, device, swapchain);
        graphicsPipeline = new GraphicsPipelineWrapper(
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
            graphicsPipeline,
            commandPool
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

    private void OnRender(double deltaTime)
    {
        synchronizedQueueSubmitterAndPresenter.OnRender();

        // TODO fix event handler stuff to make it possible to make new command queues easily?
        // eventHandler.OnRender(new(this), TimeSpan.FromSeconds(deltaTime));
    }

    private void OnClosing()
    {
        log.LogInformation("Window closing");
        Cleanup();
    }

    private void OnResize(Vector2D<int> size)
    {
        // TODO resize
    }

    private void Cleanup()
    {
        if (isCleanupDone)
        {
            return;
        }
        isCleanupDone = true;

        eventHandler.OnUnload(new(this));

        synchronizedQueueSubmitterAndPresenter.Dispose();
        commandPool.Dispose();
        graphicsPipeline.Dispose();
        renderPass.Dispose();
        swapchain.Dispose();
        device.Dispose();
        surface.Dispose();
        debugMessenger.Dispose();
        instance.Dispose();
        vk.Dispose();
    }
}
