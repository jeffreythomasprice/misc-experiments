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
        graphicsPipeline?.Dispose();
        graphicsPipeline = null;
    }

    public void OnUnload(App.State state)
    {
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

        state.Vk.CmdBindVertexBuffers(
            commandBuffer.CommandBuffer,
            0,
            1,
            [vertexBuffer.Buffer],
            [0]
        );

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
        if (uniformBuffer is null || uniformDescriptorSet is null)
        {
            throw new InvalidOperationException("not initialized");
        }

        state.Vk.DeviceWaitIdle(state.Device.Device);

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
