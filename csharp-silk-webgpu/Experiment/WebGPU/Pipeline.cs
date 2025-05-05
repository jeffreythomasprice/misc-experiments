namespace Experiment.WebGPU;
using Silk.NET.Maths;
using Silk.NET.WebGPU;

public class ShaderDescription
{
	public required string Source { get; init; }
	public required string VertexEntryPoint { get; init; }
	public required string FragmentEntryPoint { get; init; }
}

public abstract unsafe class Pipeline<T> : IDisposable where T : unmanaged
{
	private const int ProjectionMatrixBinding = 0;
	private const int ModelviewMatrixBinding = 0;

	public unsafe class ModelviewMatrix : IDisposable
	{
		private readonly VideoDriver videoDriver;
		private readonly Buffer<Matrix4X4<float>> buffer;
		private readonly BindGroup* bindGroup;

		internal ModelviewMatrix(VideoDriver videoDriver, Buffer<Matrix4X4<float>> buffer, BindGroup* bindGroup)
		{
			this.videoDriver = videoDriver;
			this.buffer = buffer;
			this.bindGroup = bindGroup;
		}

		internal BindGroup* BindGroup => bindGroup;

		public void Dispose()
		{
			buffer.Dispose();
			videoDriver.WebGPU.BindGroupRelease(bindGroup);
		}

		public void QueueWrite(Matrix4X4<float> m)
		{
			buffer.QueueWrite([m], 0);
		}
	}

	protected readonly VideoDriver videoDriver;
	private readonly Buffer<Matrix4X4<float>> projectionMatrixBuffer;
	private readonly BindGroupLayout* modelviewMatrixBindGroupLayout;
	private readonly RenderPipeline* renderPipeline;
	private readonly BindGroup* projectionMatrixBindGroup;

	protected Pipeline(VideoDriver videoDriver, ShaderDescription shaderDescription, BindGroupLayout*[] extraBindLayouts)
	{
		this.videoDriver = videoDriver;

		projectionMatrixBuffer = new Buffer<Matrix4X4<float>>(videoDriver, [Matrix4X4<float>.Identity], BufferUsage.Uniform);

		var shaderModule = videoDriver.CreateShader(shaderDescription);
		var projectionMatrixBindGroupLayout = videoDriver.CreateBindGroupLayout([
			new()
			{
				Binding = ProjectionMatrixBinding,
				Visibility = ShaderStage.Vertex,
				Buffer = new()
				{
					Type = BufferBindingType.Uniform,
				},
			},
		]);
		modelviewMatrixBindGroupLayout = videoDriver.CreateBindGroupLayout([
			new()
			{
				Binding = ModelviewMatrixBinding,
				Visibility = ShaderStage.Vertex,
				Buffer = new()
				{
					Type = BufferBindingType.Uniform,
				},
			},
		]);
		var allLayouts = new BindGroupLayout*[2 + extraBindLayouts.Length];
		allLayouts[0] = projectionMatrixBindGroupLayout;
		allLayouts[1] = modelviewMatrixBindGroupLayout;
		extraBindLayouts.CopyTo(allLayouts, 2);
		var pipelineLayout = videoDriver.CreatePipelineLayout(allLayouts);
		var vertexDescription = VertexDescription.Create<T>();
		renderPipeline = videoDriver.CreateRenderPipeline(
			GetType().FullName,
			shaderModule,
			shaderDescription.VertexEntryPoint,
			shaderDescription.FragmentEntryPoint,
			pipelineLayout,
			vertexDescription
		);

		projectionMatrixBindGroup = videoDriver.CreateBindGroup(
			projectionMatrixBindGroupLayout,
			[
				new()
				{
					Binding = ProjectionMatrixBinding,
					Buffer = projectionMatrixBuffer.Instance,
					Size = (ulong)projectionMatrixBuffer.SizeInBytes,
				},
			]
		);

		videoDriver.WebGPU.PipelineLayoutRelease(pipelineLayout);
		videoDriver.WebGPU.BindGroupLayoutRelease(projectionMatrixBindGroupLayout);
		videoDriver.WebGPU.ShaderModuleRelease(shaderModule);
	}

	public void Dispose()
	{
		projectionMatrixBuffer.Dispose();
		videoDriver.WebGPU.BindGroupLayoutRelease(modelviewMatrixBindGroupLayout);
		videoDriver.WebGPU.RenderPipelineRelease(renderPipeline);
		videoDriver.WebGPU.BindGroupRelease(projectionMatrixBindGroup);
	}

	public void QueueWriteProjectionMatrix(Matrix4X4<float> m)
	{
		projectionMatrixBuffer.QueueWrite([m], 0);
	}

	public ModelviewMatrix CreateModelviewMatrix()
	{
		var buffer = new Buffer<Matrix4X4<float>>(videoDriver, [Matrix4X4<float>.Identity], BufferUsage.Uniform);
		return new ModelviewMatrix(
			videoDriver,
			buffer,
			videoDriver.CreateBindGroup(
				modelviewMatrixBindGroupLayout,
				[
					new()
					{
						Binding = ModelviewMatrixBinding,
						Buffer = buffer.Instance,
						Size = (ulong)buffer.SizeInBytes,
					},
				]
			)
		);
	}

	protected void DrawCommon(RenderPassEncoder* renderPassEncoder, ModelviewMatrix modelviewMatrix)
	{
		videoDriver.WebGPU.RenderPassEncoderSetPipeline(renderPassEncoder, renderPipeline);
		videoDriver.WebGPU.RenderPassEncoderSetBindGroup(
			renderPassEncoder,
			0,
			projectionMatrixBindGroup,
			0,
			null
		);
		videoDriver.WebGPU.RenderPassEncoderSetBindGroup(
			renderPassEncoder,
			1,
			modelviewMatrix.BindGroup,
			0,
			null
		);
	}
}
