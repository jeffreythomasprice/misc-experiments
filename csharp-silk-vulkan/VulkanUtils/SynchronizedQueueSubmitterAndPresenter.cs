namespace Experiment.VulkanUtils;

using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using Experiment.VulkanUtils;
using Microsoft.Extensions.DependencyModel;
using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Core.Native;
using Silk.NET.Maths;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.EXT;
using Silk.NET.Vulkan.Extensions.KHR;
using Silk.NET.Windowing;

public sealed unsafe class SynchronizedQueueSubmitterAndPresenter : IDisposable
{
    // TODO configurable?
    private const int MAX_FRAMES_IN_FLIGHT = 2;

    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly SwapchainWrapper swapchain;
    private readonly RenderPassWrapper renderPass;
    private readonly GraphicsPipelineWrapper graphicsPipeline;
    private readonly CommandPoolWrapper commandPool;

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
        CommandPoolWrapper commandPool
    )
    {
        this.vk = vk;
        this.device = device;
        this.swapchain = swapchain;
        this.renderPass = renderPass;
        this.graphicsPipeline = graphicsPipeline;
        this.commandPool = commandPool;

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

        imageAvailableSemaphores = new Semaphore[MAX_FRAMES_IN_FLIGHT];
        renderFinishedSemaphores = new Semaphore[MAX_FRAMES_IN_FLIGHT];
        inFlightFences = new Fence[MAX_FRAMES_IN_FLIGHT];
        imagesInFlight = new Fence[swapchain.Images.Length];

        SemaphoreCreateInfo semaphoreInfo = new() { SType = StructureType.SemaphoreCreateInfo };

        FenceCreateInfo fenceInfo = new()
        {
            SType = StructureType.FenceCreateInfo,
            Flags = FenceCreateFlags.SignaledBit,
        };

        for (var i = 0; i < MAX_FRAMES_IN_FLIGHT; i++)
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

        for (int i = 0; i < MAX_FRAMES_IN_FLIGHT; i++)
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

        currentFrame = (currentFrame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}
