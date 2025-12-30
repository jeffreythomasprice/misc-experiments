using System;
using System.Runtime.InteropServices;
using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Core.Native;
using Silk.NET.Maths;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.EXT;
using Silk.NET.Vulkan.Extensions.KHR;
using Silk.NET.Windowing;

// TODO do logging correctly

// TODO handle resize events

// TODO handle keyboard events

// TODO next tutorial
// https://github.com/dfkeenan/SilkVulkanTutorial/blob/main/Source/06_SwapChainCreation/Program.cs

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

        // TODO props here
    }

    private readonly struct PhysicalDevicePropertiesWrapper
    {
        private readonly Vk vk;

        public readonly PhysicalDevice PhysicalDevice;

        public readonly string DeviceName;
        public readonly PhysicalDeviceType DeviceType;
        public readonly int? GraphicsQueueIndex;
        public readonly int? PresentQueueIndex;

        public PhysicalDevicePropertiesWrapper(
            Vk vk,
            KhrSurface khrSurface,
            SurfaceKHR surface,
            PhysicalDevice physicalDevice
        )
        {
            this.vk = vk;
            this.PhysicalDevice = physicalDevice;

            var properties = vk.GetPhysicalDeviceProperties(physicalDevice);

            DeviceName =
                Marshal.PtrToStringAnsi((nint)properties.DeviceName)
                ?? throw new NullReferenceException("failed to get device name");
            DeviceType = properties.DeviceType;

            uint queueFamilyPropertyCount = 0;
            vk.GetPhysicalDeviceQueueFamilyProperties(
                physicalDevice,
                ref queueFamilyPropertyCount,
                null
            );
            var queueFamilyProperties = new QueueFamilyProperties[queueFamilyPropertyCount];
            fixed (QueueFamilyProperties* ptr = queueFamilyProperties)
            {
                vk.GetPhysicalDeviceQueueFamilyProperties(
                    physicalDevice,
                    ref queueFamilyPropertyCount,
                    ptr
                );
            }

            foreach (var (property, index) in queueFamilyProperties.Select((x, i) => (x, i)))
            {
                if (
                    !GraphicsQueueIndex.HasValue
                    && property.QueueFlags.HasFlag(QueueFlags.GraphicsBit)
                )
                {
                    GraphicsQueueIndex = index;
                }
                if (!PresentQueueIndex.HasValue)
                {
                    khrSurface.GetPhysicalDeviceSurfaceSupport(
                        physicalDevice,
                        (uint)index,
                        surface,
                        out var presentSupport
                    );
                    if (presentSupport)
                    {
                        PresentQueueIndex = index;
                    }
                }
            }
        }

        public override string ToString()
        {
            return $"PhysicalDevice(Name={DeviceName}, Type={DeviceType}, GraphicsQueueIndex={GraphicsQueueIndex})";
        }
    }

    private static readonly IReadOnlyList<string> validationLayers =
    [
        "VK_LAYER_KHRONOS_validation",
    ];

    private readonly IAppEventHandler eventHandler;

    private readonly IWindow window;

    private readonly Vk vk;
    private readonly Instance instance;
    private readonly ExtDebugUtils? debugUtils;
    private readonly DebugUtilsMessengerEXT? debugMessenger;
    private readonly KhrSurface khrSurface;
    private readonly SurfaceKHR surface;
    private readonly PhysicalDevicePropertiesWrapper physicalDevice;
    private readonly Device device;
    private readonly Queue graphicsQueue;
    private readonly Queue presentQueue;

    // TODO swapchain stuff
    //     private KhrSwapchain? khrSwapChain;
    // private SwapchainKHR swapChain;
    // private Image[]? swapChainImages;
    // private Format swapChainImageFormat;
    // private Extent2D swapChainExtent;

    private bool isCleanupDone = false;

    public App(IAppEventHandler eventHandler)
    {
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
        (vk, instance) = CreateInstance(window.VkSurface, enableValidationLayers);
        (debugUtils, debugMessenger) = CreateDebugMessenger(vk, instance, enableValidationLayers);
        (khrSurface, surface) = CreateSurface(window.VkSurface, vk, instance);
        physicalDevice = FindBestPhysicalDevice(vk, instance, khrSurface, surface);
        (device, graphicsQueue) = CreateDevice(
            vk,
            khrSurface,
            surface,
            physicalDevice.PhysicalDevice,
            enableValidationLayers
        );
        // TODO CreateSwapchain()

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
        // TODO pre prender

        eventHandler.OnRender(new(this), TimeSpan.FromSeconds(deltaTime));

        // TODO post render
    }

    private void OnClosing()
    {
        Console.WriteLine("Window closing");
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

        // TODO  khrSwapChain!.DestroySwapchain(device, swapChain, null);

        vk.DestroyDevice(device, null);

        if (debugUtils != null && debugMessenger != null)
        {
            debugUtils.DestroyDebugUtilsMessenger(instance, debugMessenger.Value, null);
        }

        khrSurface.DestroySurface(instance, surface, null);

        vk?.DestroyInstance(instance, null);
        vk?.Dispose();
    }

    private static (Vk, Instance) CreateInstance(IVkSurface surface, bool enableValidationLayers)
    {
        var vk = Vk.GetApi();

        if (enableValidationLayers && !IsValidationLayerSupported(vk))
        {
            throw new Exception("validation layers requested, but not available!");
        }

        ApplicationInfo appInfo = new()
        {
            SType = StructureType.ApplicationInfo,
            PApplicationName = (byte*)Marshal.StringToHGlobalAnsi("Experiment"),
            ApplicationVersion = new Version32(1, 0, 0),
            PEngineName = (byte*)Marshal.StringToHGlobalAnsi("Experiment"),
            EngineVersion = new Version32(1, 0, 0),
            ApiVersion = Vk.Version12,
        };
        try
        {
            InstanceCreateInfo createInfo = new()
            {
                SType = StructureType.InstanceCreateInfo,
                PApplicationInfo = &appInfo,
            };
            try
            {
                var extensions = GetInstanceExtensions(surface, enableValidationLayers);
                createInfo.EnabledExtensionCount = (uint)extensions.Length;
                createInfo.PpEnabledExtensionNames = (byte**)
                    SilkMarshal.StringArrayToPtr(extensions);
                if (enableValidationLayers)
                {
                    createInfo.EnabledLayerCount = (uint)validationLayers.Count;
                    createInfo.PpEnabledLayerNames = (byte**)
                        SilkMarshal.StringArrayToPtr(validationLayers);

                    DebugUtilsMessengerCreateInfoEXT debugCreateInfo = new();
                    PopulateDebugMessengerCreateInfo(ref debugCreateInfo);
                    createInfo.PNext = &debugCreateInfo;
                }
                else
                {
                    createInfo.EnabledLayerCount = 0;
                    createInfo.PNext = null;
                }

                if (vk.CreateInstance(in createInfo, null, out var instance) != Result.Success)
                {
                    throw new Exception("failed to create instance!");
                }

                return (vk, instance);
            }
            finally
            {
                SilkMarshal.Free((nint)createInfo.PpEnabledExtensionNames);
                SilkMarshal.Free((nint)createInfo.PpEnabledLayerNames);
            }
        }
        finally
        {
            Marshal.FreeHGlobal((IntPtr)appInfo.PApplicationName);
            Marshal.FreeHGlobal((IntPtr)appInfo.PEngineName);
        }
    }

    private static (ExtDebugUtils?, DebugUtilsMessengerEXT?) CreateDebugMessenger(
        Vk vk,
        Instance instance,
        bool enableValidationLayers
    )
    {
        if (!enableValidationLayers)
        {
            return (null, null);
        }

        //TryGetInstanceExtension equivilant to method CreateDebugUtilsMessengerEXT from original tutorial.
        if (!vk.TryGetInstanceExtension(instance, out ExtDebugUtils debugUtils))
        {
            return (null, null);
        }

        DebugUtilsMessengerCreateInfoEXT createInfo = new();
        PopulateDebugMessengerCreateInfo(ref createInfo);

        if (
            debugUtils.CreateDebugUtilsMessenger(
                instance,
                in createInfo,
                null,
                out var debugMessenger
            ) != Result.Success
        )
        {
            throw new Exception("failed to set up debug messenger!");
        }

        return (debugUtils, debugMessenger);
    }

    private static (KhrSurface khrSurface, SurfaceKHR) CreateSurface(
        IVkSurface windowSurface,
        Vk vk,
        Instance instance
    )
    {
        if (!vk.TryGetInstanceExtension<KhrSurface>(instance, out var khrSurface))
        {
            throw new NotSupportedException("KHR_surface extension not found.");
        }
        var surface = windowSurface
            .Create<AllocationCallbacks>(instance.ToHandle(), null)
            .ToSurface();
        return (khrSurface, surface);
    }

    private static PhysicalDevicePropertiesWrapper FindBestPhysicalDevice(
        Vk vk,
        Instance instance,
        KhrSurface khrSurface,
        SurfaceKHR surface
    )
    {
        var devices = vk.GetPhysicalDevices(instance);
        var bestDevice = devices
            .Select(d =>
            {
                var result = new PhysicalDevicePropertiesWrapper(vk, khrSurface, surface, d);
                Console.WriteLine($"physical device: {result}");
                return result;
            })
            .Where(d => d.GraphicsQueueIndex.HasValue)
            .OrderBy(d =>
                d.DeviceType switch
                {
                    PhysicalDeviceType.DiscreteGpu => 0,
                    PhysicalDeviceType.IntegratedGpu => 1,
                    PhysicalDeviceType.Cpu => 2,
                    _ => 3,
                }
            )
            .First();

        Console.WriteLine($"Selected device: {bestDevice})");
        return bestDevice;
    }

    private static (Device Device, Queue GraphicsQueue) CreateDevice(
        Vk vk,
        KhrSurface khrSurface,
        SurfaceKHR surface,
        PhysicalDevice physicalDevice,
        bool enableValidationLayers
    )
    {
        var physicalDeviceProperties = new PhysicalDevicePropertiesWrapper(
            vk,
            khrSurface,
            surface,
            physicalDevice
        );

        if (!physicalDeviceProperties.GraphicsQueueIndex.HasValue)
        {
            throw new Exception("selected physical device has no graphics queue");
        }

        var queueCreateInfo = new DeviceQueueCreateInfo()
        {
            SType = StructureType.DeviceQueueCreateInfo,
            QueueFamilyIndex = (uint)physicalDeviceProperties.GraphicsQueueIndex.Value,
            QueueCount = 1,
        };

        var queuePriority = 1.0f;
        queueCreateInfo.PQueuePriorities = &queuePriority;

        var deviceFeatures = new PhysicalDeviceFeatures();

        var extensions = GetDeviceExtensions();
        var createInfo = new DeviceCreateInfo()
        {
            SType = StructureType.DeviceCreateInfo,
            QueueCreateInfoCount = 1,
            PQueueCreateInfos = &queueCreateInfo,
            PEnabledFeatures = &deviceFeatures,
            EnabledExtensionCount = (uint)extensions.Length,
            PpEnabledExtensionNames = (byte**)SilkMarshal.StringArrayToPtr(extensions),
        };
        try
        {
            if (enableValidationLayers)
            {
                createInfo.EnabledLayerCount = (uint)validationLayers.Count;
                createInfo.PpEnabledLayerNames = (byte**)
                    SilkMarshal.StringArrayToPtr(validationLayers);
            }
            else
            {
                createInfo.EnabledLayerCount = 0;
            }

            if (
                vk.CreateDevice(physicalDevice, in createInfo, null, out var device)
                != Result.Success
            )
            {
                throw new Exception("failed to create logical device!");
            }

            vk.GetDeviceQueue(
                device,
                (uint)physicalDeviceProperties.GraphicsQueueIndex.Value,
                0,
                out var graphicsQueue
            );

            return (device, graphicsQueue);
        }
        finally
        {
            SilkMarshal.Free((nint)createInfo.PpEnabledLayerNames);
        }
    }

    // TODO impl
    // private void CreateSwapChain()
    // {
    //     var swapChainSupport = QuerySwapChainSupport(physicalDevice);

    //     var surfaceFormat = ChooseSwapSurfaceFormat(swapChainSupport.Formats);
    //     var presentMode = ChoosePresentMode(swapChainSupport.PresentModes);
    //     var extent = ChooseSwapExtent(swapChainSupport.Capabilities);

    //     var imageCount = swapChainSupport.Capabilities.MinImageCount + 1;
    //     if (swapChainSupport.Capabilities.MaxImageCount > 0 && imageCount > swapChainSupport.Capabilities.MaxImageCount)
    //     {
    //         imageCount = swapChainSupport.Capabilities.MaxImageCount;
    //     }

    //     SwapchainCreateInfoKHR creatInfo = new()
    //     {
    //         SType = StructureType.SwapchainCreateInfoKhr,
    //         Surface = surface,

    //         MinImageCount = imageCount,
    //         ImageFormat = surfaceFormat.Format,
    //         ImageColorSpace = surfaceFormat.ColorSpace,
    //         ImageExtent = extent,
    //         ImageArrayLayers = 1,
    //         ImageUsage = ImageUsageFlags.ColorAttachmentBit,
    //     };

    //     var indices = FindQueueFamilies(physicalDevice);
    //     var queueFamilyIndices = stackalloc[] { indices.GraphicsFamily!.Value, indices.PresentFamily!.Value };

    //     if (indices.GraphicsFamily != indices.PresentFamily)
    //     {
    //         creatInfo = creatInfo with
    //         {
    //             ImageSharingMode = SharingMode.Concurrent,
    //             QueueFamilyIndexCount = 2,
    //             PQueueFamilyIndices = queueFamilyIndices,
    //         };
    //     }
    //     else
    //     {
    //         creatInfo.ImageSharingMode = SharingMode.Exclusive;
    //     }

    //     creatInfo = creatInfo with
    //     {
    //         PreTransform = swapChainSupport.Capabilities.CurrentTransform,
    //         CompositeAlpha = CompositeAlphaFlagsKHR.OpaqueBitKhr,
    //         PresentMode = presentMode,
    //         Clipped = true,

    //         OldSwapchain = default
    //     };

    //     if (!vk!.TryGetDeviceExtension(instance, device, out khrSwapChain))
    //     {
    //         throw new NotSupportedException("VK_KHR_swapchain extension not found.");
    //     }

    //     if (khrSwapChain!.CreateSwapchain(device, in creatInfo, null, out swapChain) != Result.Success)
    //     {
    //         throw new Exception("failed to create swap chain!");
    //     }

    //     khrSwapChain.GetSwapchainImages(device, swapChain, ref imageCount, null);
    //     swapChainImages = new Image[imageCount];
    //     fixed (Image* swapChainImagesPtr = swapChainImages)
    //     {
    //         khrSwapChain.GetSwapchainImages(device, swapChain, ref imageCount, swapChainImagesPtr);
    //     }

    //     swapChainImageFormat = surfaceFormat.Format;
    //     swapChainExtent = extent;
    // }

    private static bool IsValidationLayerSupported(Vk vk)
    {
        uint layerCount = 0;
        vk.EnumerateInstanceLayerProperties(ref layerCount, null);
        var availableLayers = new LayerProperties[layerCount];
        fixed (LayerProperties* availableLayersPtr = availableLayers)
        {
            vk.EnumerateInstanceLayerProperties(ref layerCount, availableLayersPtr);
        }

        var availableLayerNames = availableLayers
            .Select(layer => Marshal.PtrToStringAnsi((IntPtr)layer.LayerName))
            .ToHashSet();
        Console.WriteLine($"available layer names: [{string.Join(", ", availableLayerNames)}]");
        Console.WriteLine(
            $"looking for validation layers: [{string.Join(", ", validationLayers)}]"
        );

        return validationLayers.All(availableLayerNames.Contains);
    }

    private static string[] GetInstanceExtensions(IVkSurface surface, bool enableValidationLayers)
    {
        var glfwExtensions = surface.GetRequiredExtensions(out var glfwExtensionCount);
        var extensions = SilkMarshal.PtrToStringArray(
            (nint)glfwExtensions,
            (int)glfwExtensionCount
        );
        var result = extensions.ToList();
        if (enableValidationLayers)
        {
            result.Add(ExtDebugUtils.ExtensionName);
        }
        Console.WriteLine($"required instance extensions: [{string.Join(", ", result)}]");
        return [.. result];
    }

    private static string[] GetDeviceExtensions()
    {
        string[] result = [KhrSwapchain.ExtensionName];
        Console.WriteLine($"required device extensions: [{string.Join(", ", result)}]");
        return result;
    }

    private static void PopulateDebugMessengerCreateInfo(
        ref DebugUtilsMessengerCreateInfoEXT createInfo
    )
    {
        createInfo.SType = StructureType.DebugUtilsMessengerCreateInfoExt;
        createInfo.MessageSeverity =
            DebugUtilsMessageSeverityFlagsEXT.VerboseBitExt
            | DebugUtilsMessageSeverityFlagsEXT.WarningBitExt
            | DebugUtilsMessageSeverityFlagsEXT.ErrorBitExt;
        createInfo.MessageType =
            DebugUtilsMessageTypeFlagsEXT.GeneralBitExt
            | DebugUtilsMessageTypeFlagsEXT.PerformanceBitExt
            | DebugUtilsMessageTypeFlagsEXT.ValidationBitExt;
        createInfo.PfnUserCallback = (DebugUtilsMessengerCallbackFunctionEXT)DebugCallback;
    }

    private static uint DebugCallback(
        DebugUtilsMessageSeverityFlagsEXT messageSeverity,
        DebugUtilsMessageTypeFlagsEXT messageTypes,
        DebugUtilsMessengerCallbackDataEXT* pCallbackData,
        void* pUserData
    )
    {
        Console.WriteLine(
            $"vulkan debug callback:" + Marshal.PtrToStringAnsi((nint)pCallbackData->PMessage)
        );
        return Vk.False;
    }

    // TODO impl
    // private SurfaceFormatKHR ChooseSwapSurfaceFormat(IReadOnlyList<SurfaceFormatKHR> availableFormats)
    // {
    //     foreach (var availableFormat in availableFormats)
    //     {
    //         if (availableFormat.Format == Format.B8G8R8A8Srgb && availableFormat.ColorSpace == ColorSpaceKHR.SpaceSrgbNonlinearKhr)
    //         {
    //             return availableFormat;
    //         }
    //     }

    //     return availableFormats[0];
    // }

    // TODO impl
    // private PresentModeKHR ChoosePresentMode(IReadOnlyList<PresentModeKHR> availablePresentModes)
    // {
    //     foreach (var availablePresentMode in availablePresentModes)
    //     {
    //         if (availablePresentMode == PresentModeKHR.MailboxKhr)
    //         {
    //             return availablePresentMode;
    //         }
    //     }

    //     return PresentModeKHR.FifoKhr;
    // }

    // TODO impl
    // private Extent2D ChooseSwapExtent(SurfaceCapabilitiesKHR capabilities)
    // {
    //     if (capabilities.CurrentExtent.Width != uint.MaxValue)
    //     {
    //         return capabilities.CurrentExtent;
    //     }
    //     else
    //     {
    //         var framebufferSize = window!.FramebufferSize;

    //         Extent2D actualExtent = new()
    //         {
    //             Width = (uint)framebufferSize.X,
    //             Height = (uint)framebufferSize.Y
    //         };

    //         actualExtent.Width = Math.Clamp(actualExtent.Width, capabilities.MinImageExtent.Width, capabilities.MaxImageExtent.Width);
    //         actualExtent.Height = Math.Clamp(actualExtent.Height, capabilities.MinImageExtent.Height, capabilities.MaxImageExtent.Height);

    //         return actualExtent;
    //     }
    // }

    // TODO impl
    // private SwapChainSupportDetails QuerySwapChainSupport(PhysicalDevice physicalDevice)
    // {
    //     var details = new SwapChainSupportDetails();

    //     khrSurface!.GetPhysicalDeviceSurfaceCapabilities(physicalDevice, surface, out details.Capabilities);

    //     uint formatCount = 0;
    //     khrSurface.GetPhysicalDeviceSurfaceFormats(physicalDevice, surface, ref formatCount, null);

    //     if (formatCount != 0)
    //     {
    //         details.Formats = new SurfaceFormatKHR[formatCount];
    //         fixed (SurfaceFormatKHR* formatsPtr = details.Formats)
    //         {
    //             khrSurface.GetPhysicalDeviceSurfaceFormats(physicalDevice, surface, ref formatCount, formatsPtr);
    //         }
    //     }
    //     else
    //     {
    //         details.Formats = Array.Empty<SurfaceFormatKHR>();
    //     }

    //     uint presentModeCount = 0;
    //     khrSurface.GetPhysicalDeviceSurfacePresentModes(physicalDevice, surface, ref presentModeCount, null);

    //     if (presentModeCount != 0)
    //     {
    //         details.PresentModes = new PresentModeKHR[presentModeCount];
    //         fixed (PresentModeKHR* formatsPtr = details.PresentModes)
    //         {
    //             khrSurface.GetPhysicalDeviceSurfacePresentModes(physicalDevice, surface, ref presentModeCount, formatsPtr);
    //         }

    //     }
    //     else
    //     {
    //         details.PresentModes = Array.Empty<PresentModeKHR>();
    //     }

    //     return details;
    // }
}
