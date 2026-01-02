using System.Numerics;
using System.Runtime.CompilerServices;
using Experiment;
using Experiment.VulkanUtils;
using Microsoft.Extensions.Logging;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.Vulkan;

var log = LoggerUtils.Factory.Value.CreateLogger<Program>();
log.LogInformation("start");

using var app = new App(
    new App.CreateOptions
    {
        Title = "Experiment",
        Size = new(1280, 720),
        FixedSize = false,
    },
    new Demo()
);
app.Run();

struct UniformBufferObject
{
    public Matrix4X4<float> Model;
    public Matrix4X4<float> View;
    public Matrix4X4<float> Projection;
}

unsafe class Demo : IAppEventHandler
{
    private readonly ILogger<Demo> log;

    private BufferWrapper<Vertex2DRgba>? vertexBuffer;
    private BufferWrapper<UInt16>? indexBuffer;
    private BufferWrapper<UniformBufferObject>? uniformBuffer;
    private DescriptorSetLayoutWrapper? uniformDescriptorSetLayout;
    private DescriptorPoolWrapper? uniformDescriptorPool;
    private DescriptorSetWrapper? uniformDescriptorSet;

    private GraphicsPipelineWrapper<Vertex2DRgba>? graphicsPipeline;

    public Demo()
    {
        log = LoggerUtils.Factory.Value.CreateLogger<Demo>();
    }

    public void OnLoad(App.State state)
    {
        log.LogDebug("TODO Demo OnLoad");

        vertexBuffer = new BufferWrapper<Vertex2DRgba>(
            state.Vk,
            state.PhysicalDevice,
            state.Device,
            [
                new(new Vector2D<float>(50, 300), new Vector4D<float>(1.0f, 0.0f, 1.0f, 1.0f)),
                new(new Vector2D<float>(300, 300), new Vector4D<float>(0.0f, 0.0f, 1.0f, 1.0f)),
                new(new Vector2D<float>(300, 50), new Vector4D<float>(0.0f, 1.0f, 0.0f, 1.0f)),
                new(new Vector2D<float>(50, 50), new Vector4D<float>(1.0f, 0.0f, 0.0f, 1.0f)),
            ],
            BufferUsageFlags.VertexBufferBit
        );
        indexBuffer = new BufferWrapper<UInt16>(
            state.Vk,
            state.PhysicalDevice,
            state.Device,
            [0, 1, 2, 2, 3, 0],
            BufferUsageFlags.IndexBufferBit
        );
        log.LogInformation(
            "TODO JEFF window size = {Width}x{Height}",
            state.WindowSize.X,
            state.WindowSize.Y
        );
        uniformBuffer = new BufferWrapper<UniformBufferObject>(
            state.Vk,
            state.PhysicalDevice,
            state.Device,
            [CreateUniformBufferObject(state)],
            BufferUsageFlags.UniformBufferBit
        );
        uniformDescriptorSetLayout = new DescriptorSetLayoutWrapper(
            state.Vk,
            state.Device,
            [
                new()
                {
                    Binding = 0,
                    DescriptorCount = 1,
                    DescriptorType = DescriptorType.UniformBuffer,
                    PImmutableSamplers = null,
                    StageFlags = ShaderStageFlags.VertexBit,
                },
            ]
        );
        uniformDescriptorPool = new DescriptorPoolWrapper(
            state.Vk,
            state.Device,
            [new() { Type = DescriptorType.UniformBuffer, DescriptorCount = 1 }],
            1
        );
        uniformDescriptorSet = new DescriptorSetWrapper(
            state.Vk,
            state.Device,
            uniformDescriptorPool,
            uniformDescriptorSetLayout
        );
        uniformDescriptorSet.UpdateDescriptorSet(uniformBuffer, 0);
    }

    public void OnSwapchainCreated(App.GraphicsReadyState state)
    {
        log.LogDebug("TODO Demo OnSwapchainCreated");

        if (uniformDescriptorSetLayout is null)
        {
            throw new InvalidOperationException("not initialized");
        }

        graphicsPipeline = new GraphicsPipelineWrapper<Vertex2DRgba>(
            state.Vk,
            state.Device,
            state.Swapchain,
            state.RenderPass,
            File.ReadAllBytes("Shaders/shader.vert.spv"),
            File.ReadAllBytes("Shaders/shader.frag.spv"),
            [uniformDescriptorSetLayout]
        );
    }

    public void OnSwapchainDestroyed(App.GraphicsReadyState state)
    {
        log.LogDebug("TODO Demo OnSwapchainDestroyed");

        graphicsPipeline?.Dispose();
        graphicsPipeline = null;
    }

    public void OnUnload(App.State state)
    {
        log.LogDebug("TODO Demo OnUnload");

        uniformDescriptorSet?.Dispose();
        uniformDescriptorSet = null;
        uniformDescriptorPool?.Dispose();
        uniformDescriptorPool = null;
        uniformDescriptorSetLayout?.Dispose();
        uniformDescriptorSetLayout = null;
        uniformBuffer?.Dispose();
        uniformBuffer = null;
        indexBuffer?.Dispose();
        indexBuffer = null;
        vertexBuffer?.Dispose();
        vertexBuffer = null;
    }

    public void OnRender(
        App.GraphicsReadyState state,
        CommandBufferWrapper commandBuffer,
        TimeSpan deltaTime
    )
    {
        if (
            vertexBuffer is null
            || indexBuffer is null
            || uniformBuffer is null
            || uniformDescriptorSet is null
            || graphicsPipeline is null
        )
        {
            throw new InvalidOperationException("not initialized");
        }

        state.Vk.CmdBindPipeline(
            commandBuffer.CommandBuffer,
            PipelineBindPoint.Graphics,
            graphicsPipeline.GraphicsPipeline
        );

        // TODO helper method to automate offsets and draw?
        var vertexBuffers = new Silk.NET.Vulkan.Buffer[] { vertexBuffer.Buffer };
        var offsets = new ulong[] { 0 };
        fixed (ulong* offsetsPtr = offsets)
        fixed (Silk.NET.Vulkan.Buffer* vertexBuffersPtr = vertexBuffers)
        {
            state.Vk.CmdBindVertexBuffers(
                commandBuffer.CommandBuffer,
                0,
                1,
                vertexBuffersPtr,
                offsetsPtr
            );
        }

        state.Vk.CmdBindIndexBuffer(
            commandBuffer.CommandBuffer,
            indexBuffer.Buffer,
            0,
            IndexType.Uint16
        );

        state.Vk.CmdBindDescriptorSets(
            commandBuffer.CommandBuffer,
            PipelineBindPoint.Graphics,
            graphicsPipeline.PipelineLayout,
            0,
            1,
            in uniformDescriptorSet.DescriptorSet,
            0,
            null
        );
        state.Vk.CmdDrawIndexed(commandBuffer.CommandBuffer, (uint)indexBuffer.Count, 1, 0, 0, 0);
    }

    public void OnResize(App.State state)
    {
        log.LogDebug("TODO Demo OnResize");

        if (uniformBuffer is null || uniformDescriptorSet is null)
        {
            throw new InvalidOperationException("not initialized");
        }

        /*
        TODO fix vulkan warnings when redoing uniform buffer
        2026-01-02T18:08:36-05:00 fail: Experiment.VulkanUtils.DebugMessengerWrapper[0] vulkan debug callback Validation Error: [ VUID-vkBindBufferMemory-buffer-07459 ] Object 0: handle = 0x80000000008, type = VK_OBJECT_TYPE_DEVICE_MEMORY; Object 1: handle = 0x70000000007, type = VK_OBJECT_TYPE_BUFFER; Object 2: handle = 0x80000000008, type = VK_OBJECT_TYPE_DEVICE_MEMORY; | MessageID = 0x5001937c | vkBindBufferMemory():  attempting to bind VkDeviceMemory 0x80000000008[] to VkBuffer 0x70000000007[] which has already been bound to VkDeviceMemory 0x80000000008[]. The Vulkan spec states: buffer must not have been bound to a memory object (https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#VUID-vkBindBufferMemory-buffer-07459)
        2026-01-02T18:08:36-05:00 fail: Experiment.VulkanUtils.DebugMessengerWrapper[0] vulkan debug callback Validation Error: [ VUID-vkUpdateDescriptorSets-None-03047 ] Object 0: handle = 0xb000000000b, type = VK_OBJECT_TYPE_DESCRIPTOR_SET; Object 1: handle = 0x90000000009, type = VK_OBJECT_TYPE_DESCRIPTOR_SET_LAYOUT; | MessageID = 0x35d7ea98 | vkUpdateDescriptorSets(): pDescriptorWrites[0].dstBinding (0) was created with VkDescriptorBindingFlags(0), but VkDescriptorSet 0xb000000000b[] is in use by VkCommandBuffer 0x586b03afcea0[]. The Vulkan spec states: The dstSet member of each element of pDescriptorWrites or pDescriptorCopies for bindings which were created without the VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT or VK_DESCRIPTOR_BINDING_UPDATE_UNUSED_WHILE_PENDING_BIT bits set must not be used by any command that was recorded to a command buffer which is in the pending state (https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#VUID-vkUpdateDescriptorSets-None-03047)
        */
        uniformBuffer.CopyDataToBuffer([CreateUniformBufferObject(state)]);
        uniformDescriptorSet.UpdateDescriptorSet(uniformBuffer, 0);
    }

    public void OnKeyUp(App.State state, IKeyboard keyboard, Key key, int keyCode)
    {
        if (key == Key.Escape)
        {
            state.Exit();
        }
    }

    private UniformBufferObject CreateUniformBufferObject(App.State state)
    {
        return new UniformBufferObject
        {
            Model = Matrix4X4<float>.Identity,
            View = Matrix4X4<float>.Identity,
            Projection = GetOrthoMatrix(state),
        };
    }

    private Matrix4X4<float> GetOrthoMatrix(App.State state) =>
        Matrix4X4.CreateOrthographicOffCenter<float>(
            0,
            state.WindowSize.X,
            state.WindowSize.Y,
            0,
            -1,
            1
        );
}
