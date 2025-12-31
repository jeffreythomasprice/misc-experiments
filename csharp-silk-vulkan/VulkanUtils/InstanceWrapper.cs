namespace Experiment.VulkanUtils;

using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Vulkan;

public sealed unsafe class InstanceWrapper : IDisposable
{
    private readonly Vk vk;
    public readonly bool EnableValidationLayers;
    public readonly Instance Instance;

    public InstanceWrapper(Vk vk, IVkSurface surface, bool enableValidationLayers)
    {
        this.vk = vk;
        this.EnableValidationLayers = enableValidationLayers;

        if (
            enableValidationLayers
            && !vk.AreAllLayersSupported(DebugMessengerWrapper.REQUIRED_VALIDATION_LAYERS)
        )
        {
            throw new Exception("validation layers requested, but not available");
        }

        using var applicationName = new PointerUtils.DisposableStringPointer("Experiment");
        using var engineName = new PointerUtils.DisposableStringPointer("Experiment");
        var appInfo = new ApplicationInfo()
        {
            SType = StructureType.ApplicationInfo,
            PApplicationName = (byte*)applicationName.Pointer,
            ApplicationVersion = new Version32(1, 0, 0),
            PEngineName = (byte*)engineName.Pointer,
            EngineVersion = new Version32(1, 0, 0),
            ApiVersion = Vk.Version12,
        };

        var createInfo = new InstanceCreateInfo()
        {
            SType = StructureType.InstanceCreateInfo,
            PApplicationInfo = &appInfo,
        };

        using var extensions = new PointerUtils.DisposableStringArrayPointer(
            ExtensionsUtils.GetRequiredInstanceExtensions(surface, enableValidationLayers)
        );
        createInfo.EnabledExtensionCount = (uint)extensions.Value.Count;
        createInfo.PpEnabledExtensionNames = (byte**)extensions.Pointer;

        using var validationLayers = new PointerUtils.DisposableStringArrayPointer(
            DebugMessengerWrapper.REQUIRED_VALIDATION_LAYERS
        );
        if (enableValidationLayers)
        {
            createInfo.EnabledLayerCount = (uint)validationLayers.Value.Count;
            createInfo.PpEnabledLayerNames = (byte**)validationLayers.Pointer;

            var debugCreateInfo = DebugMessengerWrapper.CreateDebugMessengerCreateInfo();
            createInfo.PNext = &debugCreateInfo;
        }
        else
        {
            createInfo.EnabledLayerCount = 0;
            createInfo.PNext = null;
        }

        if (vk.CreateInstance(in createInfo, null, out Instance) != Result.Success)
        {
            throw new Exception("failed to create instance");
        }
    }

    public void Dispose()
    {
        vk.DestroyInstance(Instance, null);
    }
}
