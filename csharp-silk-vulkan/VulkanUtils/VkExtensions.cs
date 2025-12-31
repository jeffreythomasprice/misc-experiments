using System.Runtime.InteropServices;
using Silk.NET.Vulkan;

namespace Experiment.VulkanUtils;

public static class VkExtensions
{
    public static unsafe bool AreAllLayersSupported(this Vk vk, IEnumerable<string> requiredLayers)
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
        // TODO proper logging
        Console.WriteLine($"available layer names: [{string.Join(", ", availableLayerNames)}]");
        Console.WriteLine($"looking for required layers: [{string.Join(", ", requiredLayers)}]");

        return requiredLayers.All(availableLayerNames.Contains);
    }
}
