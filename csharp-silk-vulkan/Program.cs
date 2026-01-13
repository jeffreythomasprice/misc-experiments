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
    private TextureImageWrapper? texture;
    private Renderer2D? renderer2D;

    // OnSwapchainCreated stuff
    // ... nothing here

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

        renderer2D = new Renderer2D(state.Vk, state.Shaderc, state.PhysicalDevice, state.Device)
        {
            Texture = texture,
        };
    }

    public void OnSwapchainCreated(App.GraphicsReadyState state)
    {
        // nothing to do
    }

    public void OnSwapchainDestroyed(App.GraphicsReadyState state)
    {
        renderer2D?.OnSwapchainDestroyed();
    }

    public void OnUnload(App.State state)
    {
        renderer2D?.Dispose();
        renderer2D = null;
        texture?.Dispose();
        texture = null;
        mesh?.Dispose();
        mesh = null;
    }

    public unsafe void OnRender(
        App.GraphicsReadyState state,
        CommandBufferWrapper commandBuffer,
        TimeSpan deltaTime
    )
    {
        if (mesh is null || renderer2D is null)
        {
            throw new InvalidOperationException("not initialized");
        }

        renderer2D.Bind(state.Swapchain, state.RenderPass, commandBuffer);

        mesh.BindAndDraw(commandBuffer);
    }

    public void OnResize(App.State state)
    {
        if (renderer2D is null)
        {
            throw new InvalidOperationException("not initialized");
        }

        state.Vk.DeviceWaitIdle(state.Device.Device);

        renderer2D.ProjectionMatrix = CreateOrthoMatrix(state);
    }

    public void OnKeyUp(App.State state, IKeyboard keyboard, Key key, int keyCode)
    {
        if (key == Key.Escape)
        {
            state.Exit();
        }
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
