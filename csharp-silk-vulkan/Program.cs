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

struct UniformMatrices
{
    public Matrix4X4<float> Model;
    public Matrix4X4<float> View;
    public Matrix4X4<float> Projection;
}

unsafe class Demo : IAppEventHandler
{
    private const uint UNIFORM_MATRICES_BINDING = 0;
    private const uint UNIFORM_SAMPLER_BINDING = 1;

    private readonly ILogger<Demo> log;

    // OnLoad stuff
    private BufferWrapper<Vertex2DTexturedRgba>? vertexBuffer;
    private BufferWrapper<UInt16>? indexBuffer;
    private BufferWrapper<UniformMatrices>? uniformBuffer;
    private DescriptorSetLayoutWrapper? uniformDescriptorSetLayout;
    private DescriptorPoolWrapper? uniformDescriptorPool;
    private DescriptorSetWrapper? uniformDescriptorSet;
    private TextureImageWrapper? texture;

    // OnSwapchainCreated stuff
    private GraphicsPipelineWrapper<Vertex2DTexturedRgba>? graphicsPipeline;

    public Demo()
    {
        log = LoggerUtils.Factory.Value.CreateLogger<Demo>();
    }

    public void OnLoad(App.State state)
    {
        vertexBuffer = new BufferWrapper<Vertex2DTexturedRgba>(
            state.Vk,
            state.PhysicalDevice,
            state.Device,
            [
                new(new(50, 50), new(0, 0), new(1, 0, 0, 1)),
                new(new(50, 300), new(0, 1), new(1, 0, 1, 1)),
                new(new(300, 300), new(1, 1), new(0, 0, 1, 1)),
                new(new(300, 50), new(1, 0), new(0, 1, 0, 1)),
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
        uniformBuffer = new BufferWrapper<UniformMatrices>(
            state.Vk,
            state.PhysicalDevice,
            state.Device,
            [
                // initial blank ortho matrix, we'll set it up to an initial correct value once resize is called
                new(),
            ],
            BufferUsageFlags.UniformBufferBit
        );
        uniformDescriptorSetLayout = new DescriptorSetLayoutWrapper(
            state.Vk,
            state.Device,
            [
                new()
                {
                    Binding = UNIFORM_MATRICES_BINDING,
                    DescriptorCount = 1,
                    DescriptorType = DescriptorType.UniformBuffer,
                    PImmutableSamplers = null,
                    StageFlags = ShaderStageFlags.VertexBit,
                },
                new()
                {
                    Binding = UNIFORM_SAMPLER_BINDING,
                    DescriptorCount = 1,
                    DescriptorType = DescriptorType.CombinedImageSampler,
                    PImmutableSamplers = null,
                    StageFlags = ShaderStageFlags.FragmentBit,
                },
            ]
        );
        uniformDescriptorPool = new DescriptorPoolWrapper(
            state.Vk,
            state.Device,
            [
                new() { Type = DescriptorType.UniformBuffer, DescriptorCount = 1 },
                new() { Type = DescriptorType.CombinedImageSampler, DescriptorCount = 1 },
            ],
            1
        );
        uniformDescriptorSet = new DescriptorSetWrapper(
            state.Vk,
            state.Device,
            uniformDescriptorPool,
            uniformDescriptorSetLayout
        );

        using var sourceImage =
            SixLabors.ImageSharp.Image.Load<SixLabors.ImageSharp.PixelFormats.Rgba32>(
                "Resources/silk.png"
            );
        log.LogTrace(
            "loaded image size: {Width}x{Height}, bits per pixel: {BitsPerPixel}, alpha: {Alpha}",
            sourceImage.Width,
            sourceImage.Height,
            sourceImage.PixelType.BitsPerPixel,
            sourceImage.PixelType.AlphaRepresentation
        );
        texture = new TextureImageWrapper(
            state.Vk,
            state.PhysicalDevice,
            state.Device,
            state.CommandPool,
            sourceImage
        );
        log.LogTrace("created texture image");
        uniformDescriptorSet.UpdateDescriptorSet(texture, 1);
    }

    public void OnSwapchainCreated(App.GraphicsReadyState state)
    {
        if (uniformDescriptorSetLayout is null || uniformDescriptorSet is null)
        {
            throw new InvalidOperationException("not initialized");
        }

        graphicsPipeline = new GraphicsPipelineWrapper<Vertex2DTexturedRgba>(
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
        texture?.Dispose();
        texture = null;
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

        uniformBuffer.CopyDataToBuffer([CreateUniformMatrices(state)]);
        uniformDescriptorSet.UpdateDescriptorSet(uniformBuffer, UNIFORM_MATRICES_BINDING);
    }

    public void OnKeyUp(App.State state, IKeyboard keyboard, Key key, int keyCode)
    {
        if (key == Key.Escape)
        {
            state.Exit();
        }
    }

    private static UniformMatrices CreateUniformMatrices(App.State state)
    {
        return new UniformMatrices
        {
            Model = Matrix4X4<float>.Identity,
            View = Matrix4X4<float>.Identity,
            Projection = CreateOrthoMatrix(state),
        };
    }

    private static Matrix4X4<float> CreateOrthoMatrix(App.State state) =>
        // TODO 0,0 is in the bottom left corner, all attempts so far to make it the top-left corner have failed
        Matrix4X4.CreateOrthographicOffCenter<float>(
            0,
            state.WindowSize.X,
            state.WindowSize.Y,
            0,
            -1,
            1
        );
}
