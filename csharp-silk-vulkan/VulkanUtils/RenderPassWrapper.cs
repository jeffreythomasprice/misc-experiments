namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class RenderPassWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    public readonly RenderPass RenderPass;

    public RenderPassWrapper(Vk vk, DeviceWrapper device, SwapchainWrapper swapchain)
    {
        this.vk = vk;
        this.device = device;

        var colorAttachment = new AttachmentDescription()
        {
            Format = swapchain.Format,
            Samples = SampleCountFlags.Count1Bit,
            LoadOp = AttachmentLoadOp.Clear,
            StoreOp = AttachmentStoreOp.Store,
            StencilLoadOp = AttachmentLoadOp.DontCare,
            InitialLayout = ImageLayout.Undefined,
            FinalLayout = ImageLayout.PresentSrcKhr,
        };

        var colorAttachmentRef = new AttachmentReference()
        {
            Attachment = 0,
            Layout = ImageLayout.ColorAttachmentOptimal,
        };

        var subpass = new SubpassDescription()
        {
            PipelineBindPoint = PipelineBindPoint.Graphics,
            ColorAttachmentCount = 1,
            PColorAttachments = &colorAttachmentRef,
        };

        var dependency = new SubpassDependency()
        {
            SrcSubpass = Vk.SubpassExternal,
            DstSubpass = 0,
            SrcStageMask = PipelineStageFlags.ColorAttachmentOutputBit,
            SrcAccessMask = 0,
            DstStageMask = PipelineStageFlags.ColorAttachmentOutputBit,
            DstAccessMask = AccessFlags.ColorAttachmentWriteBit,
        };

        var renderPassInfo = new RenderPassCreateInfo()
        {
            SType = StructureType.RenderPassCreateInfo,
            AttachmentCount = 1,
            PAttachments = &colorAttachment,
            SubpassCount = 1,
            PSubpasses = &subpass,
            DependencyCount = 1,
            PDependencies = &dependency,
        };

        if (
            vk.CreateRenderPass(device.Device, in renderPassInfo, null, out RenderPass)
            != Result.Success
        )
        {
            throw new Exception("failed to create render pass");
        }
    }

    public void Dispose()
    {
        vk.DestroyRenderPass(device.Device, RenderPass, null);
    }
}
