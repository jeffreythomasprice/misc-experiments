namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class FramebufferWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly ImageViewWrapper imageView;
    private readonly bool ownsImageView;
    public readonly Framebuffer Framebuffer;

    public FramebufferWrapper(
        Vk vk,
        DeviceWrapper device,
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        ImageViewWrapper imageView,
        bool ownsImageView = false
    )
    {
        this.vk = vk;
        this.device = device;
        this.imageView = imageView;
        this.ownsImageView = ownsImageView;

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
                throw new Exception("failed to create framebuffer");
            }
        }
    }

    public FramebufferWrapper(
        Vk vk,
        DeviceWrapper device,
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        Image image
    )
        : this(
            vk,
            device,
            swapchain,
            renderPass,
            new ImageViewWrapper(vk, device, swapchain.Format, image),
            true
        ) { }

    public void Dispose()
    {
        vk.DestroyFramebuffer(device.Device, Framebuffer, null);
        if (ownsImageView)
        {
            imageView.Dispose();
        }
    }
}
