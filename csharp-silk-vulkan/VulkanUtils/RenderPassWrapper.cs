namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class RenderPassWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    public readonly RenderPass RenderPass;

    public RenderPassWrapper(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        SwapchainWrapper swapchain
    )
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

        var depthAttachment = new AttachmentDescription()
        {
            Format = DepthImageWrapper.FindBestFormat(vk, physicalDevice),
            Samples = SampleCountFlags.Count1Bit,
            LoadOp = AttachmentLoadOp.Clear,
            StoreOp = AttachmentStoreOp.DontCare,
            StencilLoadOp = AttachmentLoadOp.DontCare,
            StencilStoreOp = AttachmentStoreOp.DontCare,
            InitialLayout = ImageLayout.Undefined,
            FinalLayout = ImageLayout.DepthStencilAttachmentOptimal,
        };

        var colorAttachmentRef = new AttachmentReference()
        {
            Attachment = 0,
            Layout = ImageLayout.ColorAttachmentOptimal,
        };

        var depthAttachmentRef = new AttachmentReference()
        {
            Attachment = 1,
            Layout = ImageLayout.DepthStencilAttachmentOptimal,
        };

        var subpass = new SubpassDescription()
        {
            PipelineBindPoint = PipelineBindPoint.Graphics,
            ColorAttachmentCount = 1,
            PColorAttachments = &colorAttachmentRef,
            PDepthStencilAttachment = &depthAttachmentRef,
        };

        var dependency = new SubpassDependency()
        {
            SrcSubpass = Vk.SubpassExternal,
            DstSubpass = 0,
            SrcStageMask =
                PipelineStageFlags.ColorAttachmentOutputBit
                // TODO EarlyFragmentTestsBit is only required if depth testing is enabled, should be configurable
                | PipelineStageFlags.EarlyFragmentTestsBit,
            SrcAccessMask = 0,
            DstStageMask =
                PipelineStageFlags.ColorAttachmentOutputBit
                // TODO EarlyFragmentTestsBit is only required if depth testing is enabled, should be configurable
                | PipelineStageFlags.EarlyFragmentTestsBit,
            DstAccessMask =
                AccessFlags.ColorAttachmentWriteBit
                // TODO DepthStencilAttachmentWriteBit is only required if depth testing is enabled, should be configurable
                | AccessFlags.DepthStencilAttachmentWriteBit,
        };

        var attachments = new[] { colorAttachment, depthAttachment };
        fixed (AttachmentDescription* attachmentsPtr = attachments)
        {
            var renderPassInfo = new RenderPassCreateInfo()
            {
                SType = StructureType.RenderPassCreateInfo,
                AttachmentCount = (uint)attachments.Length,
                PAttachments = attachmentsPtr,
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
    }

    public void Dispose()
    {
        vk.DestroyRenderPass(device.Device, RenderPass, null);
    }
}
