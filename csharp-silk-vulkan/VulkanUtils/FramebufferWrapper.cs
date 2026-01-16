namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class FramebufferWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly ImageViewWrapper imageView;
    private readonly ImageViewWrapper depthImageView;
    private readonly bool ownsImageView;
    private readonly bool ownsDepthImageView;
    public readonly Framebuffer Framebuffer;

    public FramebufferWrapper(
        Vk vk,
        DeviceWrapper device,
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        ImageViewWrapper imageView,
        ImageViewWrapper depthImageView,
        bool ownsImageView = false,
        bool ownsDepthImageView = false
    )
    {
        this.vk = vk;
        this.device = device;
        this.imageView = imageView;
        this.depthImageView = depthImageView;
        this.ownsImageView = ownsImageView;
        this.ownsDepthImageView = ownsDepthImageView;

        var attachments = new[] { imageView.ImageView, depthImageView.ImageView };
        fixed (ImageView* attachmentsPtr = attachments)
        {
            var framebufferInfo = new FramebufferCreateInfo()
            {
                SType = StructureType.FramebufferCreateInfo,
                RenderPass = renderPass.RenderPass,
                AttachmentCount = (uint)attachments.Length,
                PAttachments = attachmentsPtr,
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
        Image image,
        DepthImageWrapper depthImage
    )
        : this(
            vk,
            device,
            swapchain,
            renderPass,
            new ImageViewWrapper(vk, device, swapchain.Format, image),
            new ImageViewWrapper(vk, device, depthImage.Format, depthImage.Image.Image),
            true,
            true
        ) { }

    public void Dispose()
    {
        vk.DestroyFramebuffer(device.Device, Framebuffer, null);
        if (ownsImageView)
        {
            imageView.Dispose();
        }
        if (ownsDepthImageView)
        {
            depthImageView.Dispose();
        }
    }
}
