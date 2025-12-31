namespace Experiment.VulkanUtils;

using System;
using System.Reflection.Metadata;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using Experiment.VulkanUtils;
using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Core.Native;
using Silk.NET.Maths;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.EXT;
using Silk.NET.Vulkan.Extensions.KHR;
using Silk.NET.Windowing;

public sealed unsafe class CommandBufferWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly CommandPoolWrapper commandPool;
    private readonly CommandBuffer commandBuffer;

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

        fixed (CommandBuffer* commandBufferPtr = &commandBuffer)
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

        if (vk.BeginCommandBuffer(commandBuffer, in beginInfo) != Result.Success)
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

        vk.CmdBeginRenderPass(commandBuffer, &renderPassInfo, SubpassContents.Inline);
        vk.CmdBindPipeline(
            commandBuffer,
            PipelineBindPoint.Graphics,
            graphicsPipeline.GraphicsPipeline
        );
        vk.CmdDraw(commandBuffer, 3, 1, 0, 0);
        vk.CmdEndRenderPass(commandBuffer);
        if (vk.EndCommandBuffer(commandBuffer) != Result.Success)
        {
            throw new Exception("failed to record command buffer");
        }
    }

    public void Dispose()
    {
        fixed (CommandBuffer* commandBufferPtr = &commandBuffer)
        {
            vk.FreeCommandBuffers(device.Device, commandPool.CommandPool, 1, commandBufferPtr);
        }
    }
}
