using System.Numerics;
using System.Runtime.CompilerServices;
using Experiment;
using Experiment.Engine;
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

class Demo : IAppEventHandler
{
    private const uint UNIFORM_MATRICES_BINDING = 0;
    private const uint UNIFORM_SAMPLER_BINDING = 1;

    private readonly ILogger<Demo> log;

    // OnLoad stuff
    private Mesh<Vertex2DTexturedRgba>? mesh;
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
        mesh = new Mesh<Vertex2DTexturedRgba>(state.Vk, state.PhysicalDevice, state.Device);
        mesh.AppendQuad(
            new(new(50, 50), new(0, 0), System.Drawing.Color.Red.ToVector4Df()),
            new(new(300, 50), new(1, 0), System.Drawing.Color.Green.ToVector4Df()),
            new(new(300, 300), new(1, 1), System.Drawing.Color.Blue.ToVector4Df()),
            new(new(50, 300), new(0, 1), System.Drawing.Color.Purple.ToVector4Df())
        );
        mesh.AppendQuad(
            new(new(300, 300), new(0, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(400, 300), new(1, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(400, 400), new(1, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(300, 400), new(0, 1), System.Drawing.Color.White.ToVector4Df())
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
        uniformDescriptorSet.UpdateDescriptorSet(texture, UNIFORM_SAMPLER_BINDING);
    }

    public void OnSwapchainCreated(App.GraphicsReadyState state)
    {
        if (uniformDescriptorSetLayout is null)
        {
            throw new InvalidOperationException("not initialized");
        }

        using var vertexShaderModule = ShaderModuleWrapper.FromGlslSource(
            state.Vk,
            state.Shaderc,
            state.Device,
            ShaderModuleWrapper.ShaderType.Vertex,
            """
            #version 450

            layout(binding = 0) uniform UniformMatrices {
                mat4 model;
                mat4 view;
                mat4 projection;
            } uniformMatrices;

            layout(location = 0) in vec2 inPosition;
            layout(location = 1) in vec2 inTextureCoordinate;
            layout(location = 2) in vec4 inColor;

            layout(location = 0) out vec2 fragTextureCoordinate;
            layout(location = 1) out vec4 fragColor;

            void main() {
                gl_Position = uniformMatrices.projection * uniformMatrices.view * uniformMatrices.model * vec4(inPosition, 0.0, 1.0);
                fragTextureCoordinate = inTextureCoordinate;
                fragColor = inColor;
            }
            """
        );
        using var fragmentShaderModule = ShaderModuleWrapper.FromGlslSource(
            state.Vk,
            state.Shaderc,
            state.Device,
            ShaderModuleWrapper.ShaderType.Fragment,
            """
            #version 450

            layout(binding = 1) uniform sampler2D uniformSampler;

            layout(location = 0) in vec2 fragTextureCoordinate;
            layout(location = 1) in vec4 fragColor;

            layout(location = 0) out vec4 outColor;

            void main() {
                outColor = texture(uniformSampler, fragTextureCoordinate) * fragColor;
            }
            """
        );
        graphicsPipeline = new GraphicsPipelineWrapper<Vertex2DTexturedRgba>(
            state.Vk,
            state.Device,
            state.Swapchain,
            state.RenderPass,
            vertexShaderModule,
            fragmentShaderModule,
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
        mesh?.Dispose();
        mesh = null;
    }

    public unsafe void OnRender(
        App.GraphicsReadyState state,
        CommandBufferWrapper commandBuffer,
        TimeSpan deltaTime
    )
    {
        if (
            mesh is null
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

        mesh.BindAndDraw(commandBuffer);
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
        Matrix4X4.CreateOrthographicOffCenter<float>(
            0,
            state.WindowSize.X,
            state.WindowSize.Y,
            0,
            -1,
            1
        );
}
