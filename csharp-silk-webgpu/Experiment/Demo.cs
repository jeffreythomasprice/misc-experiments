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

	private readonly Pipeline<Vertex> pipeline;
	private readonly Pipeline<Vertex>.ModelviewMatrix modelviewMatrix;
	private readonly Buffer<Vertex> vertexBuffer;
	private readonly Buffer<UInt16> indexBuffer;

	private float rotation;

	public Demo(IWindowState windowState)
	{
		this.windowState = windowState;
		this.videoDriver = (Experiment.WebGPU.VideoDriver)windowState.VideoDriver;

		unsafe
		{
			using var image = App.EmbeddedFileAsStream("Experiment.Assets.silknet.png");
			Console.WriteLine($"TODO got image as stream");

			pipeline = new(
				videoDriver,
				new()
				{
					Source = App.EmbeddedFileAsString("Experiment.Assets.Shaders.shader.wgsl"),
					VertexEntryPoint = "vs_main",
					FragmentEntryPoint = "fs_main",
				}
			);

			modelviewMatrix = pipeline.CreateModelviewMatrix();

			vertexBuffer = new(
				videoDriver,
				[
					new(
						new(-150.0f, -150.0f),
						System.Drawing.Color.Red.ToVector()
					),
					new(
						new(150.0f, -150.0f),
						System.Drawing.Color.Green.ToVector()
					),
					new(
						new(150.0f, 150.0f),
						System.Drawing.Color.Blue.ToVector()
					),
					new(
						new(-150.0f, 150.0f),
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

	public void Load()
	{
		rotation = 0;
	}

	public void Unload()
	{
		pipeline.Dispose();
		modelviewMatrix.Dispose();
		vertexBuffer.Dispose();
		indexBuffer.Dispose();
	}

	public void Resize(Vector2D<int> size)
	{
		pipeline.QueueWriteProjectionMatrix(Matrix4X4.CreateOrthographicOffCenter<float>(0, size.X, size.Y, 0, -1, 1));
	}

	public void Render()
	{
		unsafe
		{
			modelviewMatrix.QueueWrite(
				Matrix4X4.CreateRotationZ(rotation)
				* Matrix4X4.CreateTranslation(windowState.Size.X * 0.5f, windowState.Size.Y * 0.5f, 0)
			);

			videoDriver.RenderPass((renderPass) =>
			{
				pipeline.DrawBuffers(renderPass.RenderPassEncoder, modelviewMatrix, vertexBuffer, indexBuffer, 0, (uint)indexBuffer.Length);
			});
		}
	}

	public AppStateTransition? Update(TimeSpan delta)
	{
		rotation = (rotation + float.DegreesToRadians(90) * (float)delta.TotalSeconds) % float.Tau;
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
