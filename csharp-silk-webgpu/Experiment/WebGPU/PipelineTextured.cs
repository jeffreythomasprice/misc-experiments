namespace Experiment.WebGPU;

using System.Reflection;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

public unsafe class PipelineTextured : Pipeline<PipelineTextured.Vertex>, Texture.IDescriptionSource
{
	private const int TextureBinding = 0;
	private const int SamplerBinding = 1;

	public struct Vertex
	{
		[Experiment.WebGPU.VertexAttribute(Format = VertexFormat.Float32x2, ShaderLocation = 0)]
		public readonly Vector2D<float> Position;

		[Experiment.WebGPU.VertexAttribute(Format = VertexFormat.Float32x4, ShaderLocation = 1)]
		public readonly Vector2D<float> TextureCoordinate;

		[Experiment.WebGPU.VertexAttribute(Format = VertexFormat.Float32x4, ShaderLocation = 2)]
		public readonly Vector4D<float> Color;

		public Vertex(Vector2D<float> position, Vector2D<float> textureCoordinate, Vector4D<float> color)
		{
			this.Position = position;
			this.TextureCoordinate = textureCoordinate;
			this.Color = color;
		}
	}

	public PipelineTextured(VideoDriver videoDriver) : base(
			videoDriver,
			new()
			{
				Source = Assembly.GetExecutingAssembly().AssertManifestResourceString("Experiment/Assets/Shaders/shaderTextured.wgsl"),
				VertexEntryPoint = "vs_main",
				FragmentEntryPoint = "fs_main",
			},
			[
				CreateTexturedBindGroupLayout(videoDriver),
			]
		)
	{ }

	public Texture.Description TextureDescription => new()
	{
		VideoDriver = videoDriver,
		BindGroupLayout = CreateTexturedBindGroupLayout(videoDriver),
		ReleaseBindGroupLayout = true,
		TextureBinding = TextureBinding,
		SamplerBinding = SamplerBinding,
	};

	public void DrawBuffer(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix, Texture texture, Buffer<Vertex> vertexBuffer, uint index, uint length)
	{
		DrawCommon(renderPassEncoder, modelviewMatrix, texture);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, (ulong)(index * vertexBuffer.Stride), (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDraw(renderPassEncoder, length, 1, 0, 0);
	}

	public void DrawBuffers(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix, Texture texture, Buffer<Vertex> vertexBuffer, Buffer<UInt16> indexBuffer, uint index, uint length)
	{
		DrawCommon(renderPassEncoder, modelviewMatrix, texture);
		videoDriver.WebGPU.RenderPassEncoderSetVertexBuffer(renderPassEncoder, 0, vertexBuffer.Instance, 0, (ulong)vertexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderSetIndexBuffer(renderPassEncoder, indexBuffer.Instance, IndexFormat.Uint16, 0, (ulong)indexBuffer.SizeInBytes);
		videoDriver.WebGPU.RenderPassEncoderDrawIndexed(renderPassEncoder, length, 1, index, 0, 0);
	}

	private void DrawCommon(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix, Texture texture)
	{
		base.DrawCommon(renderPassEncoder, modelviewMatrix);
		videoDriver.WebGPU.RenderPassEncoderSetBindGroup(
			renderPassEncoder,
			2,
			texture.BindGroup,
			0,
			null
		);
	}

	private static BindGroupLayout* CreateTexturedBindGroupLayout(VideoDriver videoDriver)
	{
		return videoDriver.CreateBindGroupLayout([
			new BindGroupLayoutEntry()
			{
				Binding = TextureBinding,
				Visibility = ShaderStage.Fragment,
				Texture = new()
				{
					Multisampled = false,
					ViewDimension = TextureViewDimension.Dimension2D,
					SampleType = TextureSampleType.Float,
				}
			},
			new BindGroupLayoutEntry()
			{
				Binding = SamplerBinding,
				Visibility = ShaderStage.Fragment,
				Sampler = new()
				{
					Type = SamplerBindingType.Filtering,
				},
			},
		]);
	}
}