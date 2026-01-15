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
    private Mesh<Vertex2DTexturedRgba>? mesh2d;
    private TextureImageWrapper? textureFor2dMesh;
    private TextureImageWrapper? texturedStringTexture;
    private Mesh<Vertex2DTexturedRgba>? texturedStringMesh;
    private TextureImageWrapper? textureFor3dMesh;
    private Mesh<Vertex3DTexturedRgba>? mesh3d;

    private Renderer2D? renderer2D;
    private Renderer3D? renderer3D;

    private float rotation;

    // OnSwapchainCreated stuff
    // ... nothing here

    public Demo()
    {
        log = LoggerUtils.Factory.Value.CreateLogger<Demo>();
    }

    public void OnLoad(App.State state)
    {
        mesh2d = new Mesh<Vertex2DTexturedRgba>(state.Vk, state.PhysicalDevice, state.Device);
        mesh2d.AppendQuad(
            new(new(50, 50), new(0, 0), System.Drawing.Color.Red.ToVector4Df()),
            new(new(300, 50), new(1, 0), System.Drawing.Color.Green.ToVector4Df()),
            new(new(300, 300), new(1, 1), System.Drawing.Color.Blue.ToVector4Df()),
            new(new(50, 300), new(0, 1), System.Drawing.Color.Purple.ToVector4Df())
        );
        mesh2d.AppendQuad(
            new(new(300, 300), new(0, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(400, 300), new(1, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(400, 400), new(1, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(300, 400), new(0, 1), System.Drawing.Color.White.ToVector4Df())
        );

        textureFor2dMesh = TextureImageWrapper.LoadFromImageAtPath(
            state.Vk,
            state.PhysicalDevice,
            state.Device,
            state.CommandPool,
            "Resources/silk.png"
        );
        log.LogTrace("created texture image");

        var font = new TextureFont("Resources/IntelOneMono-Regular.ttf", 24);
        texturedStringTexture = font.DrawString(
            state.Vk,
            state.PhysicalDevice,
            state.Device,
            state.CommandPool,
            "Hello, World!"
        );
        log.LogTrace(
            "created string texture image, size {Width}x{Height}",
            texturedStringTexture.Width,
            texturedStringTexture.Height
        );

        texturedStringMesh = new Mesh<Vertex2DTexturedRgba>(
            state.Vk,
            state.PhysicalDevice,
            state.Device
        );
        texturedStringMesh.AppendQuad(
            new(new(0, 0), new(0, 0), System.Drawing.Color.White.ToVector4Df()),
            new(
                new(texturedStringTexture.Width, 0),
                new(1, 0),
                System.Drawing.Color.White.ToVector4Df()
            ),
            new(
                new(texturedStringTexture.Width, texturedStringTexture.Height),
                new(1, 1),
                System.Drawing.Color.White.ToVector4Df()
            ),
            new(
                new(0, texturedStringTexture.Height),
                new(0, 1),
                System.Drawing.Color.White.ToVector4Df()
            )
        );

        textureFor3dMesh = TextureImageWrapper.LoadFromImageAtPath(
            state.Vk,
            state.PhysicalDevice,
            state.Device,
            state.CommandPool,
            "Resources/ChatGPT Image Jan 15, 2026, 02_09_46 PM.png"
        );
        mesh3d = new Mesh<Vertex3DTexturedRgba>(state.Vk, state.PhysicalDevice, state.Device);
        mesh3d.AppendQuad(
            new(new(-1, -1, -1), new(0, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(-1, +1, -1), new(1, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(-1, +1, +1), new(1, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(-1, -1, +1), new(0, 1), System.Drawing.Color.White.ToVector4Df())
        );
        mesh3d.AppendQuad(
            new(new(+1, -1, +1), new(0, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, +1, +1), new(1, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, +1, -1), new(1, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, -1, -1), new(0, 0), System.Drawing.Color.White.ToVector4Df())
        );
        mesh3d.AppendQuad(
            new(new(-1, -1, +1), new(0, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, -1, +1), new(1, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, -1, -1), new(1, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(-1, -1, -1), new(0, 0), System.Drawing.Color.White.ToVector4Df())
        );
        mesh3d.AppendQuad(
            new(new(-1, +1, -1), new(0, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, +1, -1), new(1, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, +1, +1), new(1, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(-1, +1, +1), new(0, 1), System.Drawing.Color.White.ToVector4Df())
        );
        mesh3d.AppendQuad(
            new(new(-1, -1, -1), new(0, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, -1, -1), new(1, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, +1, -1), new(1, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(-1, +1, -1), new(0, 1), System.Drawing.Color.White.ToVector4Df())
        );
        mesh3d.AppendQuad(
            new(new(-1, +1, +1), new(0, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, +1, +1), new(1, 1), System.Drawing.Color.White.ToVector4Df()),
            new(new(+1, -1, +1), new(1, 0), System.Drawing.Color.White.ToVector4Df()),
            new(new(-1, -1, +1), new(0, 0), System.Drawing.Color.White.ToVector4Df())
        );

        renderer2D = new Renderer2D(state.Vk, state.Shaderc, state.PhysicalDevice, state.Device);
        renderer3D = new Renderer3D(state.Vk, state.Shaderc, state.PhysicalDevice, state.Device);

        rotation = 0;
    }

    public void OnSwapchainCreated(App.GraphicsReadyState state)
    {
        // nothing to do
    }

    public void OnSwapchainDestroyed(App.GraphicsReadyState state)
    {
        renderer2D?.OnSwapchainDestroyed();
        renderer3D?.OnSwapchainDestroyed();
    }

    public void OnUnload(App.State state)
    {
        renderer3D?.Dispose();
        renderer3D = null;
        renderer2D?.Dispose();
        renderer2D = null;
        mesh3d?.Dispose();
        mesh3d = null;
        textureFor3dMesh?.Dispose();
        textureFor3dMesh = null;
        texturedStringMesh?.Dispose();
        texturedStringMesh = null;
        texturedStringTexture?.Dispose();
        texturedStringTexture = null;
        textureFor2dMesh?.Dispose();
        textureFor2dMesh = null;
        mesh2d?.Dispose();
        mesh2d = null;
    }

    public void OnRender(
        App.GraphicsReadyState state,
        CommandBufferWrapper commandBuffer,
        TimeSpan deltaTime
    )
    {
        if (
            textureFor2dMesh is null
            || mesh2d is null
            || texturedStringTexture is null
            || texturedStringMesh is null
            || textureFor3dMesh is null
            || mesh3d is null
            || renderer2D is null
            || renderer3D is null
        )
        {
            throw new InvalidOperationException("not initialized");
        }

        renderer3D.Render(
            state.Swapchain,
            state.RenderPass,
            commandBuffer,
            CreatePerspectiveMatrix(state),
            callback =>
            {
                callback(
                    Matrix4X4.CreateRotationY(rotation)
                        * Matrix4X4.CreateTranslation<float>(0, 0, -6),
                    textureFor3dMesh,
                    () =>
                    {
                        mesh3d.BindAndDraw(commandBuffer);
                    }
                );
            }
        );

        renderer2D.Render(
            state.Swapchain,
            state.RenderPass,
            commandBuffer,
            CreateOrthoMatrix(state),
            callback =>
            {
                callback(
                    Matrix4X4<float>.Identity,
                    textureFor2dMesh,
                    () =>
                    {
                        mesh2d.BindAndDraw(commandBuffer);
                    }
                );

                callback(
                    Matrix4X4.CreateTranslation<float>(250, 100, 0),
                    texturedStringTexture,
                    () =>
                    {
                        texturedStringMesh.BindAndDraw(commandBuffer);
                    }
                );
            }
        );
    }

    public void OnUpdate(App.State state, TimeSpan deltaTime)
    {
        // TODO helper for wrapping
        rotation =
            (
                rotation
                +
                // TODO helper for degrees to radians
                (45.0f * MathF.PI / 180.0f) * (float)deltaTime.TotalSeconds
            ) % (2.0f * MathF.PI);
    }

    public void OnResize(App.State state)
    {
        if (renderer2D is null)
        {
            throw new InvalidOperationException("not initialized");
        }

        state.Vk.DeviceWaitIdle(state.Device.Device);

        // anything we want to do to vulkan stuff would go here
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

    private static Matrix4X4<float> CreatePerspectiveMatrix(App.State state) =>
        Matrix4X4.CreatePerspectiveFieldOfView(
            // TODO helper for degrees to radians
            45.0f * (MathF.PI / 180.0f),
            (float)state.WindowSize.X / (float)state.WindowSize.Y,
            0.1f,
            100.0f
        );
}
