using System.Runtime.InteropServices;
using Microsoft.Extensions.Logging;
using Silk.NET.Vulkan;

namespace Experiment.VulkanUtils;

public static class VkExtensions
{
    private static readonly Lazy<ILogger> log = new(() =>
        LoggerUtils.Factory.Value.CreateLogger(typeof(VkExtensions).ToString())
    );

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
        log.Value.LogDebug("available layer names: {AvailableLayerNames}", availableLayerNames);
        log.Value.LogDebug("looking for required layers: {RequiredLayers}", requiredLayers);

        return requiredLayers.All(availableLayerNames.Contains);
    }
}
