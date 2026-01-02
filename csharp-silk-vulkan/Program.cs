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

class Demo : IAppEventHandler
{
    private readonly ILogger<Demo> log;

    private BufferWrapper<Vertex2DRgba>? vertexBuffer;

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
                new(new Vector2D<float>(0.0f, -0.5f), new Vector4D<float>(1.0f, 0.0f, 0.0f, 1.0f)),
                new(new Vector2D<float>(0.5f, 0.5f), new Vector4D<float>(0.0f, 1.0f, 0.0f, 1.0f)),
                new(new Vector2D<float>(-0.5f, 0.5f), new Vector4D<float>(0.0f, 0.0f, 1.0f, 1.0f)),
            ],
            BufferUsageFlags.VertexBufferBit
        );
    }

    public void OnSwapchainCreated(App.GraphicsReadyState state)
    {
        log.LogDebug("TODO Demo OnSwapchainCreated");

        graphicsPipeline = new GraphicsPipelineWrapper<Vertex2DRgba>(
            state.Vk,
            state.Device,
            state.Swapchain,
            state.RenderPass,
            File.ReadAllBytes("Shaders/shader.vert.spv"),
            File.ReadAllBytes("Shaders/shader.frag.spv")
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

        vertexBuffer?.Dispose();
        vertexBuffer = null;
    }

    public void OnRender(
        App.GraphicsReadyState state,
        CommandBufferWrapper commandBuffer,
        TimeSpan deltaTime
    )
    {
        if (vertexBuffer is null || graphicsPipeline is null)
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
        unsafe
        {
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
        }

        state.Vk.CmdDraw(commandBuffer.CommandBuffer, (uint)vertexBuffer.Count, 1, 0, 0);
    }

    public void OnKeyUp(App.State state, IKeyboard keyboard, Key key, int keyCode)
    {
        if (key == Key.Escape)
        {
            state.Exit();
        }
    }
}
