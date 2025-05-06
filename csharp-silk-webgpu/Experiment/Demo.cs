using System;
using System.Reflection;
using Experiment.WebGPU;
using Silk.NET.Input;
using Silk.NET.Maths;
using Silk.NET.WebGPU;
using SixLabors.Fonts;

// TODO move me
public class Mesh<Vertex> : IDisposable where Vertex : unmanaged
{
	public readonly Buffer<Vertex> VertexBuffer;
	public readonly Buffer<UInt16> IndexBuffer;

	public Mesh(VideoDriver videoDriver, ReadOnlySpan<Vertex> vertices, ReadOnlySpan<UInt16> indices)
	{
		VertexBuffer = new(videoDriver, vertices, BufferUsage.Vertex);
		IndexBuffer = new(videoDriver, indices, BufferUsage.Index);
	}

	public void Dispose()
	{
		VertexBuffer.Dispose();
		IndexBuffer.Dispose();
	}
}

// TODO move me
public class Actor<Vertex> : IDisposable where Vertex : unmanaged
{
	public readonly Mesh<Vertex> Mesh;
	public readonly Pipeline.ModelviewMatrix ModelviewMatrix;

	public Actor(Pipeline<Vertex> pipeline, Mesh<Vertex> mesh)
	{
		this.Mesh = mesh;
		this.ModelviewMatrix = pipeline.CreateModelviewMatrix();
	}

	public void Dispose()
	{
		ModelviewMatrix.Dispose();
	}
}

public unsafe static class ActorExtensions
{
	public static void Draw(this PipelineUntextured pipeline, RenderPassEncoder* renderPassEncoder, Actor<PipelineUntextured.Vertex> actor)
	{
		pipeline.DrawBuffers(
			renderPassEncoder,
			actor.ModelviewMatrix,
			actor.Mesh.VertexBuffer,
			actor.Mesh.IndexBuffer,
			0,
			(uint)actor.Mesh.IndexBuffer.Length
		);
	}

	public static void Draw(this PipelineTextured pipeline, RenderPassEncoder* renderPassEncoder, Experiment.WebGPU.Texture texture, Actor<PipelineTextured.Vertex> actor)
	{
		pipeline.DrawBuffers(
			renderPassEncoder,
			actor.ModelviewMatrix,
			texture,
			actor.Mesh.VertexBuffer,
			actor.Mesh.IndexBuffer,
			0,
			(uint)actor.Mesh.IndexBuffer.Length
		);
	}
}

class Demo : IAppState
{
	private readonly IWindowState windowState;
	private readonly Experiment.WebGPU.VideoDriver videoDriver;

	private readonly PipelineUntextured untexturedPipeline;
	private readonly PipelineTextured texturedPipeline;

	private readonly Experiment.WebGPU.Texture texture;
	private readonly Experiment.WebGPU.Texture textTexture;

	private readonly Mesh<PipelineUntextured.Vertex> untexturedMesh;
	private readonly Actor<PipelineUntextured.Vertex> untexturedActor;

	private readonly Mesh<PipelineTextured.Vertex> texturedMesh;
	private readonly Actor<PipelineTextured.Vertex> texturedActor;

	private readonly Mesh<PipelineTextured.Vertex> textMesh;
	private readonly Actor<PipelineTextured.Vertex> textActor;

	private float rotation;

	public Demo(IWindowState windowState)
	{
		this.windowState = windowState;
		this.videoDriver = (Experiment.WebGPU.VideoDriver)windowState.VideoDriver;

		unsafe
		{
			untexturedPipeline = new(videoDriver);
			texturedPipeline = new(videoDriver);

			texture = texturedPipeline.CreateTextureFromManifestResource("Experiment/Assets/silknet.png");

			/*
			TODO various geometry helpers
			- make textured and untextured rectangles
			- text
			*/

			var fontCollection = new FontCollection();
			using var fontStream = Assembly.GetExecutingAssembly().AssertManifestResourceStream("Experiment/Assets/calibri-font-family/calibri-regular.ttf");
			var fontFamily = fontCollection.Add(fontStream);
			var font = fontFamily.CreateFont(40.0f);
			textTexture = texturedPipeline.CreateTextureFromString(font, "Hello, World!");

			untexturedMesh = new(
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
				[
					0,1,2,
					2,3,0,
				]
			);
			untexturedActor = new(untexturedPipeline, untexturedMesh);

			texturedMesh = new(
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
				[
					0,1,2,
					2,3,0,
				]
			);
			texturedActor = new(texturedPipeline, texturedMesh);

			textMesh = new(
				videoDriver,
				[
					new(
						new(0.0f,0.0f),
						new(0.0f,0.0f),
						System.Drawing.Color.White.ToVector()
					),
					new(
						new(textTexture.Size.X,0.0f),
						new(1.0f,0.0f),
						System.Drawing.Color.White.ToVector()
					),
					new(
						new(textTexture.Size.X,textTexture.Size.Y),
						new(1.0f,1.0f),
						System.Drawing.Color.White.ToVector()
					),
					new(
						new(0.0f,textTexture.Size.Y),
						new(0.0f,1.0f),
						System.Drawing.Color.White.ToVector()
					),
				],
				[
					0,1,2,
					2,3,0,
				]
			);
			textActor = new(texturedPipeline, textMesh);
		}
	}

	public void Load()
	{
		rotation = 0;
	}

	public void Unload()
	{
		untexturedPipeline.Dispose();
		texturedPipeline.Dispose();

		texture.Dispose();
		textTexture.Dispose();

		untexturedMesh.Dispose();
		untexturedActor.Dispose();

		texturedMesh.Dispose();
		texturedActor.Dispose();

		textMesh.Dispose();
		textActor.Dispose();
	}

	public void Resize(Vector2D<int> size)
	{
		var ortho = Matrix4X4.CreateOrthographicOffCenter<float>(0, size.X, size.Y, 0, -1, 1);
		untexturedPipeline.QueueWriteProjectionMatrix(ortho);
		texturedPipeline.QueueWriteProjectionMatrix(ortho);
	}

	public void Render()
	{
		unsafe
		{
			untexturedActor.ModelviewMatrix.QueueWrite(
				Matrix4X4.CreateRotationZ(rotation)
				* Matrix4X4.CreateTranslation(windowState.Size.X * 0.5f, windowState.Size.Y * 0.5f, 0)
			);

			texturedActor.ModelviewMatrix.QueueWrite(Matrix4X4<float>.Identity);

			textActor.ModelviewMatrix.QueueWrite(Matrix4X4.CreateTranslation(50.0f, 250.0f, 0));

			videoDriver.RenderPass((renderPass) =>
			{
				untexturedPipeline.Draw(renderPass.RenderPassEncoder, untexturedActor);
				texturedPipeline.Draw(renderPass.RenderPassEncoder, texture, texturedActor);
				texturedPipeline.Draw(renderPass.RenderPassEncoder, textTexture, textActor);
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
