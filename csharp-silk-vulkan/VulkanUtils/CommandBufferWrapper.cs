namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class CommandBufferWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly CommandPoolWrapper commandPool;
    public readonly CommandBuffer CommandBuffer;

    public static void OneTimeSubmit(
        Vk vk,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        Action<CommandBufferWrapper> callback
    )
    {
        using var commandBuffer = new CommandBufferWrapper(
            vk,
            device,
            commandPool,
            CommandBufferUsageFlags.OneTimeSubmitBit,
            (commandBuffer) =>
            {
                callback(commandBuffer);
            }
        );
        fixed (CommandBuffer* commandBufferPtr = &commandBuffer.CommandBuffer)
        {
            var submitInfo = new SubmitInfo()
            {
                SType = StructureType.SubmitInfo,
                CommandBufferCount = 1,
                PCommandBuffers = commandBufferPtr,
            };

            vk.QueueSubmit(device.GraphicsQueue, 1, in submitInfo, default);
            vk.QueueWaitIdle(device.GraphicsQueue);
        }
    }

    private CommandBufferWrapper(
        Vk vk,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        CommandBufferUsageFlags flags,
        Action<CommandBufferWrapper> callback
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
            Flags = CommandBufferUsageFlags.None,
        };

        if (vk.BeginCommandBuffer(CommandBuffer, in beginInfo) != Result.Success)
        {
            throw new Exception("failed to begin recording command buffer");
        }

        callback(this);

        if (vk.EndCommandBuffer(CommandBuffer) != Result.Success)
        {
            throw new Exception("failed to record command buffer");
        }
    }

    public CommandBufferWrapper(
        Vk vk,
        DeviceWrapper device,
        CommandPoolWrapper commandPool,
        CommandBufferUsageFlags flags,
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        FramebufferWrapper framebuffer,
        Action<CommandBufferWrapper> callback
    )
        : this(
            vk,
            device,
            commandPool,
            flags,
            (commandBuffer) =>
            {
                var renderPassInfo = new RenderPassBeginInfo()
                {
                    SType = StructureType.RenderPassBeginInfo,
                    RenderPass = renderPass.RenderPass,
                    Framebuffer = framebuffer.Framebuffer,
                    RenderArea = { Offset = { X = 0, Y = 0 }, Extent = swapchain.Extent },
                };

                var clearColor = new ClearValue()
                {
                    Color = System.Drawing.Color.CornflowerBlue.ToClearColorValue(),
                };

                renderPassInfo.ClearValueCount = 1;
                renderPassInfo.PClearValues = &clearColor;

                vk.CmdBeginRenderPass(
                    commandBuffer.CommandBuffer,
                    &renderPassInfo,
                    SubpassContents.Inline
                );

                callback(commandBuffer);

                vk.CmdEndRenderPass(commandBuffer.CommandBuffer);
            }
        ) { }

    public void Dispose()
    {
        fixed (CommandBuffer* commandBufferPtr = &CommandBuffer)
        {
            vk.FreeCommandBuffers(device.Device, commandPool.CommandPool, 1, commandBufferPtr);
        }
    }
}
