namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class SynchronizedQueueSubmitterAndPresenter : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly SwapchainWrapper swapchain;
    private readonly RenderPassWrapper renderPass;
    private readonly GraphicsPipelineWrapper graphicsPipeline;
    private readonly CommandPoolWrapper commandPool;
    private readonly int maxFramesInFlight;

    // TODO combine into a struct?
    private readonly List<ImageViewWrapper> swapchainImageViews;
    private readonly List<FramebufferWrapper> framebuffers;

    // TODO combine sync primitives into a shared struct? do they really go together in that way?
    private readonly Semaphore[] imageAvailableSemaphores;
    private readonly Semaphore[] renderFinishedSemaphores;
    private readonly Fence[] inFlightFences;
    private readonly Fence[] imagesInFlight;
    private int currentFrame = 0;

    public SynchronizedQueueSubmitterAndPresenter(
        Vk vk,
        DeviceWrapper device,
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        GraphicsPipelineWrapper graphicsPipeline,
        CommandPoolWrapper commandPool,
        int maxFramesInFlight = 2
    )
    {
        this.vk = vk;
        this.device = device;
        this.swapchain = swapchain;
        this.renderPass = renderPass;
        this.graphicsPipeline = graphicsPipeline;
        this.commandPool = commandPool;
        this.maxFramesInFlight = maxFramesInFlight;

        if (maxFramesInFlight <= 0)
        {
            throw new Exception(
                $"must provide at least one frame in flight, got {maxFramesInFlight}"
            );
        }

        swapchainImageViews =
        [
            .. swapchain.Images.Select(image => new ImageViewWrapper(
                vk,
                device,
                swapchain.Format,
                image
            )),
        ];

        framebuffers =
        [
            .. swapchainImageViews.Select(imageView => new FramebufferWrapper(
                vk,
                device,
                swapchain,
                renderPass,
                imageView
            )),
        ];

        imageAvailableSemaphores = new Semaphore[this.maxFramesInFlight];
        renderFinishedSemaphores = new Semaphore[this.maxFramesInFlight];
        inFlightFences = new Fence[this.maxFramesInFlight];
        imagesInFlight = new Fence[swapchain.Images.Length];

        SemaphoreCreateInfo semaphoreInfo = new() { SType = StructureType.SemaphoreCreateInfo };

        FenceCreateInfo fenceInfo = new()
        {
            SType = StructureType.FenceCreateInfo,
            Flags = FenceCreateFlags.SignaledBit,
        };

        for (var i = 0; i < this.maxFramesInFlight; i++)
        {
            // TODO clean up partial allocations if any fail
            if (
                vk.CreateSemaphore(
                    device.Device,
                    in semaphoreInfo,
                    null,
                    out imageAvailableSemaphores[i]
                ) != Result.Success
                || vk.CreateSemaphore(
                    device.Device,
                    in semaphoreInfo,
                    null,
                    out renderFinishedSemaphores[i]
                ) != Result.Success
                || vk.CreateFence(device.Device, in fenceInfo, null, out inFlightFences[i])
                    != Result.Success
            )
            {
                throw new Exception("failed to create synchronization objects for a frame");
            }
        }
    }

    public void Dispose()
    {
        vk.DeviceWaitIdle(device.Device);

        for (int i = 0; i < maxFramesInFlight; i++)
        {
            vk.DestroySemaphore(device.Device, renderFinishedSemaphores[i], null);
            vk.DestroySemaphore(device.Device, imageAvailableSemaphores[i], null);
            vk.DestroyFence(device.Device, inFlightFences[i], null);
        }

        foreach (var framebuffer in framebuffers)
        {
            framebuffer.Dispose();
        }

        foreach (var imageView in swapchainImageViews)
        {
            imageView.Dispose();
        }
    }

    public void OnRender()
    {
        vk.WaitForFences(device.Device, 1, in inFlightFences[currentFrame], true, ulong.MaxValue);

        uint imageIndex = 0;
        swapchain.KhrSwapchain.AcquireNextImage(
            device.Device,
            swapchain.SwapchainKhr,
            ulong.MaxValue,
            imageAvailableSemaphores[currentFrame],
            default,
            ref imageIndex
        );

        if (imagesInFlight[imageIndex].Handle != default)
        {
            vk.WaitForFences(device.Device, 1, in imagesInFlight[imageIndex], true, ulong.MaxValue);
        }
        imagesInFlight[imageIndex] = inFlightFences[currentFrame];

        var submitInfo = new SubmitInfo() { SType = StructureType.SubmitInfo };

        var waitSemaphores = stackalloc[] { imageAvailableSemaphores[currentFrame] };
        var waitStages = stackalloc[] { PipelineStageFlags.ColorAttachmentOutputBit };

        // TODO the constructor for command buffer is presumably where custom per-frame rendering instructions will go
        var commandBuffer = new CommandBufferWrapper(
            vk,
            device,
            swapchain,
            renderPass,
            graphicsPipeline,
            framebuffers[(int)imageIndex],
            commandPool
        );
        fixed (CommandBuffer* commandBufferPtr = &commandBuffer.CommandBuffer)
        {
            submitInfo = submitInfo with
            {
                WaitSemaphoreCount = 1,
                PWaitSemaphores = waitSemaphores,
                PWaitDstStageMask = waitStages,

                CommandBufferCount = 1,
                PCommandBuffers = commandBufferPtr,
            };

            var signalSemaphores = stackalloc[] { renderFinishedSemaphores[currentFrame] };
            submitInfo = submitInfo with
            {
                SignalSemaphoreCount = 1,
                PSignalSemaphores = signalSemaphores,
            };

            vk.ResetFences(device.Device, 1, in inFlightFences[currentFrame]);

            if (
                vk.QueueSubmit(device.GraphicsQueue, 1, in submitInfo, inFlightFences[currentFrame])
                != Result.Success
            )
            {
                throw new Exception("failed to submit draw command buffer");
            }

            var swapChains = stackalloc[] { swapchain.SwapchainKhr };
            PresentInfoKHR presentInfo = new()
            {
                SType = StructureType.PresentInfoKhr,

                WaitSemaphoreCount = 1,
                PWaitSemaphores = signalSemaphores,

                SwapchainCount = 1,
                PSwapchains = swapChains,

                PImageIndices = &imageIndex,
            };

            swapchain.KhrSwapchain.QueuePresent(device.PresentQueue, in presentInfo);
        }

        currentFrame = (currentFrame + 1) % maxFramesInFlight;
    }
}
