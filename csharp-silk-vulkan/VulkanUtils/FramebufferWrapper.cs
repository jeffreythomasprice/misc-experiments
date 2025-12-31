namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class FramebufferWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    public readonly Framebuffer Framebuffer;

    public FramebufferWrapper(
        Vk vk,
        DeviceWrapper device,
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        ImageViewWrapper imageView
    )
    {
        this.vk = vk;
        this.device = device;

        fixed (ImageView* attachment = &imageView.ImageView)
        {
            var framebufferInfo = new FramebufferCreateInfo()
            {
                SType = StructureType.FramebufferCreateInfo,
                RenderPass = renderPass.RenderPass,
                AttachmentCount = 1,
                PAttachments = attachment,
                Width = swapchain.Extent.Width,
                Height = swapchain.Extent.Height,
                Layers = 1,
            };

            if (
                vk.CreateFramebuffer(device.Device, in framebufferInfo, null, out Framebuffer)
                != Result.Success
            )
            {
                throw new Exception("failed to create framebuffer!");
            }
        }
    }

    public void Dispose()
    {
        vk.DestroyFramebuffer(device.Device, Framebuffer, null);
    }
}
