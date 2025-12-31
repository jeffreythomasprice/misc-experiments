using Silk.NET.Core.Contexts;
using Silk.NET.Core.Native;
using Silk.NET.Vulkan.Extensions.EXT;
using Silk.NET.Vulkan.Extensions.KHR;

public static unsafe class ExtensionsUtils
{
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
        Console.WriteLine($"required instance extensions: [{string.Join(", ", result)}]");
        return [.. result];
    }

    public static string[] GetRequiredDeviceExtensions()
    {
        string[] result = [KhrSwapchain.ExtensionName];
        Console.WriteLine($"required device extensions: [{string.Join(", ", result)}]");
        return result;
    }
}
