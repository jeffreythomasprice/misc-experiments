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
    public CommandBufferWrapper() { }

    public void Dispose()
    {
        throw new NotImplementedException();
    }

    // TODO impl

    //  private void CreateCommandBuffers()
    // {
    //     commandBuffers = new CommandBuffer[swapChainFramebuffers!.Length];

    //     CommandBufferAllocateInfo allocInfo = new()
    //     {
    //         SType = StructureType.CommandBufferAllocateInfo,
    //         CommandPool = commandPool,
    //         Level = CommandBufferLevel.Primary,
    //         CommandBufferCount = (uint)commandBuffers.Length,
    //     };

    //     fixed (CommandBuffer* commandBuffersPtr = commandBuffers)
    //     {
    //         if (vk!.AllocateCommandBuffers(device, in allocInfo, commandBuffersPtr) != Result.Success)
    //         {
    //             throw new Exception("failed to allocate command buffers!");
    //         }
    //     }

    //     for (int i = 0; i < commandBuffers.Length; i++)
    //     {
    //         CommandBufferBeginInfo beginInfo = new()
    //         {
    //             SType = StructureType.CommandBufferBeginInfo,
    //         };

    //         if (vk!.BeginCommandBuffer(commandBuffers[i], in beginInfo) != Result.Success)
    //         {
    //             throw new Exception("failed to begin recording command buffer!");
    //         }

    //         RenderPassBeginInfo renderPassInfo = new()
    //         {
    //             SType = StructureType.RenderPassBeginInfo,
    //             RenderPass = renderPass,
    //             Framebuffer = swapChainFramebuffers[i],
    //             RenderArea =
    //             {
    //                 Offset = { X = 0, Y = 0 },
    //                 Extent = swapChainExtent,
    //             }
    //         };

    //         ClearValue clearColor = new()
    //         {
    //             Color = new() { Float32_0 = 0, Float32_1 = 0, Float32_2 = 0, Float32_3 = 1 },
    //         };

    //         renderPassInfo.ClearValueCount = 1;
    //         renderPassInfo.PClearValues = &clearColor;

    //         vk!.CmdBeginRenderPass(commandBuffers[i], &renderPassInfo, SubpassContents.Inline);

    //         vk!.CmdBindPipeline(commandBuffers[i], PipelineBindPoint.Graphics, graphicsPipeline);

    //         vk!.CmdDraw(commandBuffers[i], 3, 1, 0, 0);

    //         vk!.CmdEndRenderPass(commandBuffers[i]);

    //         if (vk!.EndCommandBuffer(commandBuffers[i]) != Result.Success)
    //         {
    //             throw new Exception("failed to record command buffer!");
    //         }

    //     }
    // }
}
