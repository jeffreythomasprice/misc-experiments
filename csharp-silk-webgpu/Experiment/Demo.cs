using System;
using Experiment.WebGPU;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

struct Vertex
{
	[Experiment.WebGPU.VertexAttribute(Format = VertexFormat.Float32x2, ShaderLocation = 0)]
	public readonly Vector2D<float> Position;
	[Experiment.WebGPU.VertexAttribute(Format = VertexFormat.Float32x4, ShaderLocation = 1)]
	public readonly Vector4D<float> Color;

	public Vertex(Vector2D<float> position, Vector4D<float> color)
	{
		this.Position = position;
		this.Color = color;
	}
}

class Demo : IAppState
{
	private readonly IWindowState windowState;
	private readonly Experiment.WebGPU.VideoDriver videoDriver;

	private readonly Pipeline pipeline;
	private readonly Buffer<Vertex> vertexBuffer;
	private readonly Buffer<UInt16> indexBuffer;

	public Demo(IWindowState windowState)
	{
		this.windowState = windowState;
		this.videoDriver = (Experiment.WebGPU.VideoDriver)windowState.VideoDriver;

		unsafe
		{
			pipeline = new Pipeline(
				videoDriver,
				new()
				{
					ShaderDescription = new()
					{
						Source = App.EmbeddedFileAsString("Experiment.Assets.Shaders.shader.wgsl"),
						VertexEntryPoint = "vs_main",
						FragmentEntryPoint = "fs_main",
					},
					VertexBufferDescription = VertexBufferDescription<Vertex>.Create()
				}
			);
			vertexBuffer = new(
				videoDriver,
				[
					new(
						new(-0.5f, -0.5f),
						System.Drawing.Color.Red.ToVector()
					),
					new(
						new(0.5f, -0.5f),
						System.Drawing.Color.Green.ToVector()
					),
					new(
						new(0.5f, 0.5f),
						System.Drawing.Color.Blue.ToVector()
					),
					new(
						new(-0.5f, 0.5f),
						System.Drawing.Color.Purple.ToVector()
					),
				],
				BufferUsage.Vertex
			);
			indexBuffer = new(
				videoDriver,
				[
					0,1,2,
					2,3,0,
				],
				BufferUsage.Index
			);
		}
	}

	public void Load() { }

	public void Unload()
	{
		pipeline.Dispose();
	}

	public void Resize(Vector2D<int> size) { }

	public void Render()
	{
		unsafe
		{
			videoDriver.RenderPass((renderPass) =>
			{
				pipeline.DrawBuffers(renderPass.RenderPassEncoder, vertexBuffer, indexBuffer, 0, (uint)indexBuffer.Length);
			});
		}
	}

	public AppStateTransition? Update(TimeSpan delta)
	{
		return null;
	}

	public AppStateTransition? KeyDown(Key key)
	{
		return null;
	}

	public AppStateTransition? KeyUp(Key key)
	{
		if (key == Key.Escape)
		{
			return AppStateTransition.Exit;
		}
		return null;
	}
}
