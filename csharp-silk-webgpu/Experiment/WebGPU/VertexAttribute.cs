namespace Experiment.WebGPU;

using Silk.NET.WebGPU;

[AttributeUsage(AttributeTargets.Field)]
public class VertexAttribute : Attribute
{
	public required VertexFormat Format { get; init; }
	public required uint ShaderLocation { get; init; }
}