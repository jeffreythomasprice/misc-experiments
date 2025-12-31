namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class CommandBufferWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly CommandPoolWrapper commandPool;
    public readonly CommandBuffer CommandBuffer;

    public CommandBufferWrapper(
        Vk vk,
        DeviceWrapper device,
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        GraphicsPipelineWrapper graphicsPipeline,
        FramebufferWrapper framebuffer,
        CommandPoolWrapper commandPool
    )
    {
        this.vk = vk;
        this.device = device;
        this.commandPool = commandPool;

        var allocInfo = new CommandBufferAllocateInfo()
        {
            SType = StructureType.CommandBufferAllocateInfo,
            CommandPool = commandPool.CommandPool,
            Level = CommandBufferLevel.Primary,
            CommandBufferCount = 1,
        };

        fixed (CommandBuffer* commandBufferPtr = &CommandBuffer)
        {
            if (
                vk.AllocateCommandBuffers(device.Device, in allocInfo, commandBufferPtr)
                != Result.Success
            )
            {
                throw new Exception("failed to allocate command buffers");
            }
        }

        var beginInfo = new CommandBufferBeginInfo()
        {
            SType = StructureType.CommandBufferBeginInfo,
        };

        if (vk.BeginCommandBuffer(CommandBuffer, in beginInfo) != Result.Success)
        {
            throw new Exception("failed to begin recording command buffer");
        }

        var renderPassInfo = new RenderPassBeginInfo()
        {
            SType = StructureType.RenderPassBeginInfo,
            RenderPass = renderPass.RenderPass,
            Framebuffer = framebuffer.Framebuffer,
            RenderArea = { Offset = { X = 0, Y = 0 }, Extent = swapchain.Extent },
        };

        var clearColor = new ClearValue()
        {
            Color = new()
            {
                Float32_0 = 0,
                Float32_1 = 0,
                Float32_2 = 0,
                Float32_3 = 1,
            },
        };

        renderPassInfo.ClearValueCount = 1;
        renderPassInfo.PClearValues = &clearColor;

        vk.CmdBeginRenderPass(CommandBuffer, &renderPassInfo, SubpassContents.Inline);
        vk.CmdBindPipeline(
            CommandBuffer,
            PipelineBindPoint.Graphics,
            graphicsPipeline.GraphicsPipeline
        );
        vk.CmdDraw(CommandBuffer, 3, 1, 0, 0);
        vk.CmdEndRenderPass(CommandBuffer);

        if (vk.EndCommandBuffer(CommandBuffer) != Result.Success)
        {
            throw new Exception("failed to record command buffer");
        }
    }

    public void Dispose()
    {
        fixed (CommandBuffer* commandBufferPtr = &CommandBuffer)
        {
            vk.FreeCommandBuffers(device.Device, commandPool.CommandPool, 1, commandBufferPtr);
        }
    }
}
