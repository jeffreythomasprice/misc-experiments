using Experiment;
using Microsoft.Extensions.Logging;
using Silk.NET.Core.Contexts;
using Silk.NET.Core.Native;
using Silk.NET.Vulkan.Extensions.EXT;
using Silk.NET.Vulkan.Extensions.KHR;

public static unsafe class ExtensionsUtils
{
    private static readonly Lazy<ILogger> log = new(() =>
        LoggerUtils.Factory.Value.CreateLogger(typeof(ExtensionsUtils).ToString())
    );

    public static string[] GetRequiredInstanceExtensions(
        IVkSurface surface,
        bool enableValidationLayers
    )
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
        log.Value.LogDebug("required instance extensions {Extensions}", result);
        return [.. result];
    }

    public static string[] GetRequiredDeviceExtensions()
    {
        string[] result = [KhrSwapchain.ExtensionName];
        log.Value.LogDebug("required device extensions {Extensions}", result);
        return result;
    }
}
