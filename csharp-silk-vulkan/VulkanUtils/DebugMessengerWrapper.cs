namespace Experiment.VulkanUtils;

using System.Runtime.InteropServices;
using Microsoft.Extensions.Logging;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.EXT;

public sealed unsafe class DebugMessengerWrapper : IDisposable
{
    public static readonly IReadOnlyList<string> REQUIRED_VALIDATION_LAYERS =
    [
        "VK_LAYER_KHRONOS_validation",
    ];

    private static readonly Lazy<ILogger> Log = new(() =>
        LoggerUtils.Factory.Value.CreateLogger<DebugMessengerWrapper>()
    );

    private readonly InstanceWrapper instance;
    private readonly ExtDebugUtils? debugUtils;
    private readonly DebugUtilsMessengerEXT? debugMessenger;

    public DebugMessengerWrapper(Vk vk, InstanceWrapper instance, bool enableValidationLayers)
    {
        this.instance = instance;

        if (!enableValidationLayers)
        {
            return;
        }

        //TryGetInstanceExtension equivilant to method CreateDebugUtilsMessengerEXT from original tutorial.
        if (!vk.TryGetInstanceExtension(instance.Instance, out ExtDebugUtils debugUtils))
        {
            return;
        }
        this.debugUtils = debugUtils;

        var createInfo = CreateDebugMessengerCreateInfo();
        if (
            debugUtils.CreateDebugUtilsMessenger(
                instance.Instance,
                in createInfo,
                null,
                out var debugMessenger
            ) != Result.Success
        )
        {
            throw new Exception("failed to set up debug messenger");
        }
        this.debugMessenger = debugMessenger;
    }

    public void Dispose()
    {
        if (debugUtils != null && debugMessenger != null)
        {
            debugUtils.DestroyDebugUtilsMessenger(instance.Instance, debugMessenger.Value, null);
        }
    }

    public static DebugUtilsMessengerCreateInfoEXT CreateDebugMessengerCreateInfo()
    {
        var result = new DebugUtilsMessengerCreateInfoEXT();
        result.SType = StructureType.DebugUtilsMessengerCreateInfoExt;
        result.MessageSeverity =
            DebugUtilsMessageSeverityFlagsEXT.VerboseBitExt
            | DebugUtilsMessageSeverityFlagsEXT.WarningBitExt
            | DebugUtilsMessageSeverityFlagsEXT.ErrorBitExt;
        result.MessageType =
            DebugUtilsMessageTypeFlagsEXT.GeneralBitExt
            | DebugUtilsMessageTypeFlagsEXT.PerformanceBitExt
            | DebugUtilsMessageTypeFlagsEXT.ValidationBitExt;
        result.PfnUserCallback = (DebugUtilsMessengerCallbackFunctionEXT)DebugCallback;
        return result;
    }

    private static uint DebugCallback(
        DebugUtilsMessageSeverityFlagsEXT messageSeverity,
        DebugUtilsMessageTypeFlagsEXT messageTypes,
        DebugUtilsMessengerCallbackDataEXT* pCallbackData,
        void* pUserData
    )
    {
        var message = Marshal.PtrToStringAnsi((nint)pCallbackData->PMessage);
        switch (messageSeverity)
        {
            case DebugUtilsMessageSeverityFlagsEXT.None:
                break;
            case DebugUtilsMessageSeverityFlagsEXT.VerboseBitExt:
                Log.Value.LogTrace("vulkan debug callback {Message}", message);
                break;
            case DebugUtilsMessageSeverityFlagsEXT.InfoBitExt:
                Log.Value.LogInformation("vulkan debug callback {Message}", message);
                break;
            case DebugUtilsMessageSeverityFlagsEXT.WarningBitExt:
                Log.Value.LogWarning("vulkan debug callback {Message}", message);
                break;
            case DebugUtilsMessageSeverityFlagsEXT.ErrorBitExt:
                Log.Value.LogError("vulkan debug callback {Message}", message);
                break;
        }
        return Vk.False;
    }
}
