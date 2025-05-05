using System;
using Experiment.WebGPU;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

class Demo : IAppState
{
	private readonly IWindowState windowState;
	private readonly Experiment.WebGPU.VideoDriver videoDriver;

	private readonly PipelineUntextured pipelineUntextured;
	private readonly PipelineTextured pipelineTextured;

	private readonly PipelineTextured.Texture texture;

	private readonly PipelineUntextured.ModelviewMatrix modelviewMatrixUntextured;
	private readonly Buffer<PipelineUntextured.Vertex> vertexBufferUntextured;
	private readonly Buffer<UInt16> indexBufferUntextured;

	private readonly PipelineTextured.ModelviewMatrix modelviewMatrixTextured;
	private readonly Buffer<PipelineTextured.Vertex> vertexBufferTextured;
	private readonly Buffer<UInt16> indexBufferTextured;

	private float rotation;

	public Demo(IWindowState windowState)
	{
		this.windowState = windowState;
		this.videoDriver = (Experiment.WebGPU.VideoDriver)windowState.VideoDriver;

		unsafe
		{
			pipelineUntextured = new(videoDriver);
			pipelineTextured = new(videoDriver);

			texture = pipelineTextured.CreateTextureFromEmbeddedFile("Experiment.Assets.silknet.png");

			modelviewMatrixUntextured = pipelineUntextured.CreateModelviewMatrix();
			vertexBufferUntextured = new(
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
			indexBufferUntextured = new(
				videoDriver,
				[
					0,1,2,
					2,3,0,
				],
				BufferUsage.Index
			);

			modelviewMatrixTextured = pipelineTextured.CreateModelviewMatrix();
			vertexBufferTextured = new(
				videoDriver,
				[
					new(
						new(0.0f,0.0f),
						new(0.0f,0.0f),
						System.Drawing.Color.White.ToVector()
					),
					new(
						new(texture.Size.X,0.0f),
						new(1.0f,0.0f),
						System.Drawing.Color.White.ToVector()
					),
					new(
						new(texture.Size.X,texture.Size.Y),
						new(1.0f,1.0f),
						System.Drawing.Color.White.ToVector()
					),
					new(
						new(0.0f,texture.Size.Y),
						new(0.0f,1.0f),
						System.Drawing.Color.White.ToVector()
					),
				],
				BufferUsage.Vertex
			);
			indexBufferTextured = new(
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
		pipelineUntextured.Dispose();
		pipelineTextured.Dispose();

		texture.Dispose();

		modelviewMatrixUntextured.Dispose();
		vertexBufferUntextured.Dispose();
		indexBufferUntextured.Dispose();

		modelviewMatrixTextured.Dispose();
		vertexBufferTextured.Dispose();
		indexBufferTextured.Dispose();
	}

	public void Resize(Vector2D<int> size)
	{
		var ortho = Matrix4X4.CreateOrthographicOffCenter<float>(0, size.X, size.Y, 0, -1, 1);
		pipelineUntextured.QueueWriteProjectionMatrix(ortho);
		pipelineTextured.QueueWriteProjectionMatrix(ortho);
	}

	public void Render()
	{
		unsafe
		{
			modelviewMatrixUntextured.QueueWrite(
				Matrix4X4.CreateRotationZ(rotation)
				* Matrix4X4.CreateTranslation(windowState.Size.X * 0.5f, windowState.Size.Y * 0.5f, 0)
			);

			modelviewMatrixTextured.QueueWrite(Matrix4X4<float>.Identity);

			videoDriver.RenderPass((renderPass) =>
			{
				pipelineUntextured.DrawBuffers(renderPass.RenderPassEncoder, modelviewMatrixUntextured, vertexBufferUntextured, indexBufferUntextured, 0, (uint)indexBufferUntextured.Length);
				pipelineTextured.DrawBuffers(
					renderPass.RenderPassEncoder,
					modelviewMatrixTextured,
					texture,
					vertexBufferTextured,
					indexBufferTextured,
					0,
					(uint)indexBufferTextured.Length
				);
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
