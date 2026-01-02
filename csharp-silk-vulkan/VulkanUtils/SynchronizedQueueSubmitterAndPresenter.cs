namespace Experiment.VulkanUtils;

using System;
using Microsoft.Extensions.Logging;
using Silk.NET.Vulkan;

public sealed unsafe class SynchronizedQueueSubmitterAndPresenter : IDisposable
{
    private readonly ILogger log;

    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly SwapchainWrapper swapchain;
    private readonly RenderPassWrapper renderPass;
    private readonly CommandPoolWrapper commandPool;
    private readonly int maxFramesInFlight;

    private readonly List<FramebufferWrapper> framebuffers;

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
        CommandPoolWrapper commandPool,
        int maxFramesInFlight = 2
    )
    {
        log = LoggerUtils.Factory.Value.CreateLogger(GetType());

        this.vk = vk;
        this.device = device;
        this.swapchain = swapchain;
        this.renderPass = renderPass;
        this.commandPool = commandPool;
        this.maxFramesInFlight = maxFramesInFlight;

        if (maxFramesInFlight <= 0)
        {
            throw new Exception(
                $"must provide at least one frame in flight, got {maxFramesInFlight}"
            );
        }

        framebuffers =
        [
            .. swapchain.Images.Select(image => new FramebufferWrapper(
                vk,
                device,
                swapchain,
                renderPass,
                image
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

        try
        {
            for (var i = 0; i < this.maxFramesInFlight; i++)
            {
                if (
                    vk.CreateSemaphore(
                        device.Device,
                        in semaphoreInfo,
                        null,
                        out imageAvailableSemaphores[i]
                    ) != Result.Success
                )
                {
                    throw new Exception("failed to create semaphore");
                }

                if (
                    vk.CreateSemaphore(
                        device.Device,
                        in semaphoreInfo,
                        null,
                        out renderFinishedSemaphores[i]
                    ) != Result.Success
                )
                {
                    throw new Exception("failed to create semaphore");
                }

                if (
                    vk.CreateFence(device.Device, in fenceInfo, null, out inFlightFences[i])
                    != Result.Success
                )
                {
                    throw new Exception("failed to create fence");
                }
            }
        }
        catch
        {
            foreach (var fence in inFlightFences)
            {
                if (fence.Handle != default)
                {
                    vk.DestroyFence(device.Device, fence, null);
                }
            }

            foreach (var semaphore in renderFinishedSemaphores)
            {
                if (semaphore.Handle != default)
                {
                    vk.DestroySemaphore(device.Device, semaphore, null);
                }
            }

            foreach (var semaphore in imageAvailableSemaphores)
            {
                if (semaphore.Handle != default)
                {
                    vk.DestroySemaphore(device.Device, semaphore, null);
                }
            }

            throw;
        }
    }

    public void Dispose()
    {
        vk.DeviceWaitIdle(device.Device);

        foreach (var fence in inFlightFences)
        {
            vk.DestroyFence(device.Device, fence, null);
        }

        foreach (var semaphore in renderFinishedSemaphores)
        {
            vk.DestroySemaphore(device.Device, semaphore, null);
        }

        foreach (var semaphore in imageAvailableSemaphores)
        {
            vk.DestroySemaphore(device.Device, semaphore, null);
        }

        foreach (var framebuffer in framebuffers)
        {
            framebuffer.Dispose();
        }
    }

    /// <summary>
    ///
    /// </summary>
    /// <param name="renderPassCallback">should do stuff like CmdBindVertexBuffers and CmdDraw</param>
    /// <param name="needsRecreate">set to true if an error occurred that indicates we need to rebuild everything from swapchain on down</param>
    /// <exception cref="NotImplementedException"></exception>
    /// <exception cref="Exception"></exception>
    public void OnRender(Action<CommandBufferWrapper> renderPassCallback, out bool needsRecreate)
    {
        vk.WaitForFences(device.Device, 1, in inFlightFences[currentFrame], true, ulong.MaxValue);

        uint imageIndex = 0;
        var result = swapchain.KhrSwapchain.AcquireNextImage(
            device.Device,
            swapchain.SwapchainKhr,
            ulong.MaxValue,
            imageAvailableSemaphores[currentFrame],
            default,
            ref imageIndex
        );
        if (result == Result.ErrorOutOfDateKhr)
        {
            log.LogDebug(
                "AcquireNextImage failed with {Result}, signalling we need to recreate",
                result
            );
            needsRecreate = true;
            return;
        }
        else if (result != Result.Success && result != Result.SuboptimalKhr)
        {
            throw new Exception($"rendering failed to acquire swap chain image: {result}");
        }

        if (imagesInFlight[imageIndex].Handle != default)
        {
            vk.WaitForFences(device.Device, 1, in imagesInFlight[imageIndex], true, ulong.MaxValue);
        }
        imagesInFlight[imageIndex] = inFlightFences[currentFrame];

        var submitInfo = new SubmitInfo() { SType = StructureType.SubmitInfo };

        var waitSemaphores = stackalloc[] { imageAvailableSemaphores[currentFrame] };
        var waitStages = stackalloc[] { PipelineStageFlags.ColorAttachmentOutputBit };

        var commandBuffer = new CommandBufferWrapper(
            vk,
            device,
            swapchain,
            renderPass,
            framebuffers[(int)imageIndex],
            commandPool,
            renderPassCallback
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

            result = swapchain.KhrSwapchain.QueuePresent(device.PresentQueue, in presentInfo);
            if (result == Result.ErrorOutOfDateKhr || result == Result.SuboptimalKhr)
            {
                log.LogDebug(
                    "QueuePresent failed with {Result}, signalling we need to recreate",
                    result
                );
                needsRecreate = true;
            }
            else if (result != Result.Success)
            {
                throw new Exception($"rendering failed to present queue: {result}");
            }
        }

        currentFrame = (currentFrame + 1) % maxFramesInFlight;

        needsRecreate = false;
    }
}
