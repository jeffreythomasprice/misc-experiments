namespace Experiment.WebGPU;

using System.Reflection;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

public unsafe class PipelineUntextured : Pipeline<PipelineUntextured.Vertex>
{
	public struct Vertex
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

	public PipelineUntextured(VideoDriver videoDriver) : base(
		videoDriver,
		new()
		{
			Source = Assembly.GetExecutingAssembly().AssertManifestResourceString("Experiment/Assets/Shaders/shaderUntextured.wgsl"),
			VertexEntryPoint = "vs_main",
			FragmentEntryPoint = "fs_main",
		},
		[]
	)
	{ }

	public void DrawBuffer(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix, Buffer<Vertex> vertexBuffer, uint index, uint length)
	{
		DrawCommon(renderPassEncoder, modelviewMatrix);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, (ulong)(index * vertexBuffer.Stride), (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDraw(renderPassEncoder, length, 1, 0, 0);
	}

	public void DrawBuffers(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix, Buffer<Vertex> vertexBuffer, Buffer<UInt16> indexBuffer, uint index, uint length)
	{
		DrawCommon(renderPassEncoder, modelviewMatrix);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, 0, (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderSetIndexBuffer(renderPassEncoder, indexBuffer.Instance, IndexFormat.Uint16, 0, (ulong)indexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDrawIndexed(renderPassEncoder, length, 1, index, 0, 0);
	}
}